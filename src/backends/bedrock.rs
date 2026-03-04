//! AWS Bedrock Runtime backend.
//!
//! Sends requests to the Bedrock Runtime API with AWS SigV4 request signing
//! and converts AWS event-stream binary responses to SSE for streaming.
//!
//! # Example
//!
//! ```no_run
//! use anthropic_rs::{Anthropic, backends::{AwsCredentials, BedrockBackend}};
//!
//! # fn example() -> anthropic_rs::Result<()> {
//! let backend = BedrockBackend::builder()
//!     .credentials(AwsCredentials {
//!         access_key_id: "AKIA...".into(),
//!         secret_access_key: "secret".into(),
//!         session_token: None,
//!     })
//!     .region("us-east-1")
//!     .build()?;
//!
//! let client = Anthropic::builder().backend(backend).build()?;
//! # Ok(())
//! # }
//! ```

use std::fmt::Write as FmtWrite;

use hmac::{Hmac, Mac};
use reqwest::header::{HeaderMap, HeaderValue};
use sha2::{Digest, Sha256};

use crate::error::{AnthropicError, Result};

use super::{Backend, BackendRequest, StreamTransformer};

const BEDROCK_VERSION: &str = "bedrock-2023-05-31";
const SERVICE_NAME: &str = "bedrock";

/// AWS credentials for SigV4 request signing.
#[derive(Debug, Clone)]
pub struct AwsCredentials {
    pub access_key_id: String,
    pub secret_access_key: String,
    pub session_token: Option<String>,
}

/// Authentication method for Bedrock.
#[derive(Debug, Clone)]
enum BedrockAuth {
    /// AWS IAM credentials with SigV4 signing.
    Credentials(AwsCredentials),
    /// Bearer token (AWS SSO or cross-account).
    BearerToken(String),
}

/// Backend for AWS Bedrock Runtime.
///
/// Transforms Anthropic API requests into Bedrock-compatible format:
/// - Rewrites URL path: `/v1/messages` → `/model/{modelId}/invoke`
/// - Moves `model` and `stream` from body to URL
/// - Adds `anthropic_version` to request body
/// - Signs requests with AWS SigV4 (or uses bearer token)
/// - Converts binary event-stream responses to SSE for streaming
#[derive(Debug, Clone)]
pub struct BedrockBackend {
    auth: BedrockAuth,
    region: String,
    base_url: String,
}

impl BedrockBackend {
    /// The AWS region this backend targets.
    pub fn region(&self) -> &str {
        &self.region
    }
}

impl BedrockBackend {
    pub fn builder() -> BedrockBuilder {
        BedrockBuilder::new()
    }

    /// Create from environment variables.
    ///
    /// Reads `AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`, `AWS_SESSION_TOKEN`,
    /// `AWS_BEARER_TOKEN_BEDROCK`, and `AWS_REGION` / `AWS_DEFAULT_REGION`.
    pub fn from_env() -> Result<Self> {
        Self::builder().from_env().build()
    }
}

impl Backend for BedrockBackend {
    fn base_url(&self) -> &str {
        &self.base_url
    }

    fn prepare_request(&self, mut req: BackendRequest) -> Result<BackendRequest> {
        // Validate path segments
        if req.path_segments.is_empty() || req.path_segments[0] != "v1" {
            return Err(AnthropicError::Config(
                "Expected first path segment to be 'v1'".into(),
            ));
        }
        if req.path_segments.len() < 2 {
            return Err(AnthropicError::Config(
                "Missing service name in request URL".into(),
            ));
        }

        let service = &req.path_segments[1];
        match service.as_str() {
            "messages" => {
                if req.path_segments.len() > 2 {
                    match req.path_segments[2].as_str() {
                        "batches" => {
                            return Err(AnthropicError::Config(
                                "Batch API is not supported for Bedrock".into(),
                            ))
                        }
                        "count_tokens" => {
                            return Err(AnthropicError::Config(
                                "Token counting is not supported for Bedrock".into(),
                            ))
                        }
                        _ => {}
                    }
                }
            }
            "complete" => {}
            other => {
                return Err(AnthropicError::Config(format!(
                    "Service is not supported for Bedrock: {other}"
                )))
            }
        }

        // Transform body
        let body = req.body.as_mut().ok_or_else(|| {
            AnthropicError::InvalidData("Bedrock request has no body".into())
        })?;
        let obj = body.as_object_mut().ok_or_else(|| {
            AnthropicError::InvalidData("Bedrock request body is not an object".into())
        })?;

        // Add anthropic_version to body
        obj.insert(
            "anthropic_version".to_owned(),
            serde_json::Value::String(BEDROCK_VERSION.to_owned()),
        );

        // Move anthropic-beta header values into body
        if let Some(beta_val) = req.headers.remove("anthropic-beta") {
            let beta_str = beta_val.to_str().unwrap_or("");
            let versions: Vec<serde_json::Value> = beta_str
                .split(',')
                .map(|s| serde_json::Value::String(s.trim().to_owned()))
                .collect();
            if !versions.is_empty() {
                obj.insert(
                    "anthropic_beta".to_owned(),
                    serde_json::Value::Array(versions),
                );
            }
        }

        // Extract model ID from body
        let model_id = obj
            .remove("model")
            .and_then(|v| v.as_str().map(String::from))
            .ok_or_else(|| {
                AnthropicError::InvalidData("No model found in request body".into())
            })?;

        // Extract stream flag
        let is_stream = obj
            .remove("stream")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        // Rewrite path segments: /model/{modelId}/invoke[-with-response-stream]
        let action = if is_stream {
            "invoke-with-response-stream"
        } else {
            "invoke"
        };
        req.path_segments = vec!["model".into(), model_id, action.into()];

        // Set content-type
        req.headers
            .insert("content-type", HeaderValue::from_static("application/json"));

        Ok(req)
    }

    fn authorize_request(
        &self,
        mut req: BackendRequest,
        resolved_url: &url::Url,
    ) -> Result<BackendRequest> {
        match &self.auth {
            BedrockAuth::Credentials(creds) => {
                let body_bytes = req
                    .body
                    .as_ref()
                    .map(|b| serde_json::to_vec(b).unwrap_or_default())
                    .unwrap_or_default();

                let now = chrono::Utc::now();
                let amz_date = now.format("%Y%m%dT%H%M%SZ").to_string();
                let date_stamp = now.format("%Y%m%d").to_string();

                // Add required headers for signing
                req.headers.insert(
                    "x-amz-date",
                    HeaderValue::from_str(&amz_date)
                        .map_err(|e| AnthropicError::Config(format!("invalid date: {e}")))?,
                );
                req.headers.insert(
                    "host",
                    HeaderValue::from_str(resolved_url.host_str().unwrap_or(""))
                        .map_err(|e| AnthropicError::Config(format!("invalid host: {e}")))?,
                );

                if let Some(ref token) = creds.session_token {
                    req.headers.insert(
                        "x-amz-security-token",
                        HeaderValue::from_str(token).map_err(|e| {
                            AnthropicError::Config(format!("invalid session token: {e}"))
                        })?,
                    );
                }

                // Sign the request
                let authorization = sign_v4(&SigV4Params {
                    method: req.method.as_str(),
                    url: resolved_url,
                    headers: &req.headers,
                    body: &body_bytes,
                    credentials: creds,
                    region: &self.region,
                    service: SERVICE_NAME,
                    amz_date: &amz_date,
                    date_stamp: &date_stamp,
                });

                req.headers.insert(
                    "authorization",
                    HeaderValue::from_str(&authorization)
                        .map_err(|e| AnthropicError::Config(format!("invalid auth header: {e}")))?,
                );
            }
            BedrockAuth::BearerToken(token) => {
                req.headers.insert(
                    "authorization",
                    HeaderValue::from_str(&format!("Bearer {token}"))
                        .map_err(|e| AnthropicError::Config(format!("invalid token: {e}")))?,
                );
            }
        }

        Ok(req)
    }

    fn stream_transformer(&self) -> Option<Box<dyn StreamTransformer>> {
        Some(Box::new(EventStreamDecoder::new()))
    }
}

/// Builder for [`BedrockBackend`].
pub struct BedrockBuilder {
    credentials: Option<AwsCredentials>,
    bearer_token: Option<String>,
    region: Option<String>,
}

impl BedrockBuilder {
    fn new() -> Self {
        Self {
            credentials: None,
            bearer_token: None,
            region: None,
        }
    }

    /// Load configuration from environment variables.
    pub fn from_env(mut self) -> Self {
        // Check for bearer token first
        if let Ok(token) = std::env::var("AWS_BEARER_TOKEN_BEDROCK") {
            self.bearer_token = Some(token);
        } else {
            // Fall back to IAM credentials
            if let (Ok(key_id), Ok(secret)) = (
                std::env::var("AWS_ACCESS_KEY_ID"),
                std::env::var("AWS_SECRET_ACCESS_KEY"),
            ) {
                self.credentials = Some(AwsCredentials {
                    access_key_id: key_id,
                    secret_access_key: secret,
                    session_token: std::env::var("AWS_SESSION_TOKEN").ok(),
                });
            }
        }
        if let Ok(region) = std::env::var("AWS_REGION")
            .or_else(|_| std::env::var("AWS_DEFAULT_REGION"))
        {
            self.region = Some(region);
        }
        self
    }

    pub fn credentials(mut self, creds: AwsCredentials) -> Self {
        self.credentials = Some(creds);
        self.bearer_token = None;
        self
    }

    pub fn bearer_token(mut self, token: impl Into<String>) -> Self {
        self.bearer_token = Some(token.into());
        self.credentials = None;
        self
    }

    pub fn region(mut self, region: impl Into<String>) -> Self {
        self.region = Some(region.into());
        self
    }

    pub fn build(self) -> Result<BedrockBackend> {
        let auth = if let Some(creds) = self.credentials {
            BedrockAuth::Credentials(creds)
        } else if let Some(token) = self.bearer_token {
            BedrockAuth::BearerToken(token)
        } else {
            return Err(AnthropicError::Config(
                "AWS credentials or bearer token must be provided for Bedrock".into(),
            ));
        };

        let region = self.region.ok_or_else(|| {
            AnthropicError::Config(
                "AWS region must be set. Use AWS_REGION env var or .region() builder method."
                    .into(),
            )
        })?;

        let base_url = format!("https://bedrock-runtime.{region}.amazonaws.com");
        Ok(BedrockBackend {
            auth,
            region,
            base_url,
        })
    }
}

impl Default for BedrockBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// ── AWS SigV4 Signing ─────────────────────────────────────────────────────

type HmacSha256 = Hmac<Sha256>;

fn sha256_hex(data: &[u8]) -> String {
    let hash = Sha256::digest(data);
    let mut hex = String::with_capacity(64);
    for byte in hash {
        write!(hex, "{byte:02x}").unwrap();
    }
    hex
}

fn hmac_sha256(key: &[u8], data: &[u8]) -> Vec<u8> {
    let mut mac =
        HmacSha256::new_from_slice(key).expect("HMAC can take key of any size");
    mac.update(data);
    mac.finalize().into_bytes().to_vec()
}

fn hex_encode(data: &[u8]) -> String {
    let mut hex = String::with_capacity(data.len() * 2);
    for byte in data {
        write!(hex, "{byte:02x}").unwrap();
    }
    hex
}

/// Parameters for AWS SigV4 request signing.
struct SigV4Params<'a> {
    method: &'a str,
    url: &'a url::Url,
    headers: &'a HeaderMap,
    body: &'a [u8],
    credentials: &'a AwsCredentials,
    region: &'a str,
    service: &'a str,
    amz_date: &'a str,
    date_stamp: &'a str,
}

/// Compute the AWS SigV4 Authorization header value.
fn sign_v4(params: &SigV4Params<'_>) -> String {
    let SigV4Params {
        method,
        url,
        headers,
        body,
        credentials,
        region,
        service,
        amz_date,
        date_stamp,
    } = params;
    let scope = format!("{date_stamp}/{region}/{service}/aws4_request");

    // 1. Canonical request
    let canonical_uri = url.path();
    let canonical_querystring = url.query().unwrap_or("");

    // Collect and sort headers (lowercase names)
    let mut signed_headers_vec: Vec<(String, String)> = headers
        .iter()
        .map(|(k, v)| {
            (
                k.as_str().to_lowercase(),
                v.to_str().unwrap_or("").trim().to_owned(),
            )
        })
        .collect();
    signed_headers_vec.sort_by(|a, b| a.0.cmp(&b.0));

    let canonical_headers: String = signed_headers_vec
        .iter()
        .map(|(k, v)| format!("{k}:{v}\n"))
        .collect();
    let signed_headers: String = signed_headers_vec
        .iter()
        .map(|(k, _)| k.as_str())
        .collect::<Vec<_>>()
        .join(";");

    let payload_hash = sha256_hex(body);

    let canonical_request = format!(
        "{method}\n{canonical_uri}\n{canonical_querystring}\n{canonical_headers}\n{signed_headers}\n{payload_hash}"
    );

    // 2. String to sign
    let canonical_request_hash = sha256_hex(canonical_request.as_bytes());
    let string_to_sign = format!(
        "AWS4-HMAC-SHA256\n{amz_date}\n{scope}\n{canonical_request_hash}"
    );

    // 3. Signing key
    let k_date = hmac_sha256(
        format!("AWS4{}", credentials.secret_access_key).as_bytes(),
        date_stamp.as_bytes(),
    );
    let k_region = hmac_sha256(&k_date, region.as_bytes());
    let k_service = hmac_sha256(&k_region, service.as_bytes());
    let k_signing = hmac_sha256(&k_service, b"aws4_request");

    // 4. Signature
    let signature = hex_encode(&hmac_sha256(&k_signing, string_to_sign.as_bytes()));

    format!(
        "AWS4-HMAC-SHA256 Credential={}/{scope}, SignedHeaders={signed_headers}, Signature={signature}",
        credentials.access_key_id
    )
}

// ── AWS Event-Stream → SSE Decoder ────────────────────────────────────────

/// Decodes AWS event-stream binary frames into SSE text events.
///
/// The Bedrock streaming API returns responses in the AWS event-stream binary
/// format. Each frame contains a base64-encoded JSON payload that, once decoded,
/// is a standard Anthropic SSE event.
pub struct EventStreamDecoder {
    buf: Vec<u8>,
}

impl EventStreamDecoder {
    pub fn new() -> Self {
        Self { buf: Vec::new() }
    }
}

impl Default for EventStreamDecoder {
    fn default() -> Self {
        Self::new()
    }
}

impl StreamTransformer for EventStreamDecoder {
    fn transform(&mut self, data: &[u8], output: &mut Vec<u8>) -> Result<()> {
        use base64::Engine;

        self.buf.extend_from_slice(data);

        // Process complete frames
        while self.buf.len() >= 12 {
            // Read total byte length (first 4 bytes, big-endian)
            let total_len = u32::from_be_bytes([
                self.buf[0],
                self.buf[1],
                self.buf[2],
                self.buf[3],
            ]) as usize;

            if self.buf.len() < total_len {
                break; // Need more data for complete frame
            }

            // Read headers byte length (bytes 4-7)
            let headers_len = u32::from_be_bytes([
                self.buf[4],
                self.buf[5],
                self.buf[6],
                self.buf[7],
            ]) as usize;

            // Payload starts after prelude (12 bytes) + headers
            let payload_offset = 12 + headers_len;
            // Payload ends before the 4-byte message CRC
            let payload_end = total_len.saturating_sub(4);

            if payload_offset < payload_end {
                let payload = &self.buf[payload_offset..payload_end];

                // Parse payload JSON: {"bytes": "base64EncodedSSEData"}
                if let Ok(json) = serde_json::from_slice::<serde_json::Value>(payload) {
                    if let Some(b64) = json.get("bytes").and_then(|v| v.as_str()) {
                        if let Ok(decoded) =
                            base64::engine::general_purpose::STANDARD.decode(b64)
                        {
                            let sse_json = String::from_utf8_lossy(&decoded);

                            // Extract event type from the JSON
                            if let Ok(event_val) =
                                serde_json::from_str::<serde_json::Value>(&sse_json)
                            {
                                if let Some(event_type) =
                                    event_val.get("type").and_then(|v| v.as_str())
                                {
                                    let sse =
                                        format!("event: {event_type}\ndata: {sse_json}\n\n");
                                    output.extend_from_slice(sse.as_bytes());
                                }
                            }
                        }
                    }
                }
            }

            // Remove processed frame from buffer
            self.buf.drain(..total_len);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha256_hex_works() {
        assert_eq!(
            sha256_hex(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(
            sha256_hex(b"hello"),
            "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
        );
    }

    #[test]
    fn bedrock_prepare_request_transforms_path() {
        let backend = BedrockBackend {
            auth: BedrockAuth::BearerToken("test".into()),
            region: "us-east-1".into(),
            base_url: "https://bedrock-runtime.us-east-1.amazonaws.com".into(),
        };

        let body = serde_json::json!({
            "model": "anthropic.claude-3-sonnet-20240229-v1:0",
            "max_tokens": 1024,
            "messages": [{"role": "user", "content": "Hello"}]
        });

        let req = BackendRequest {
            method: reqwest::Method::POST,
            path_segments: vec!["v1".into(), "messages".into()],
            query_params: vec![],
            headers: HeaderMap::new(),
            body: Some(body),
        };

        let prepared = backend.prepare_request(req).unwrap();

        assert_eq!(prepared.path_segments, vec!["model", "anthropic.claude-3-sonnet-20240229-v1:0", "invoke"]);

        let body = prepared.body.unwrap();
        assert!(body.get("model").is_none()); // model removed from body
        assert_eq!(body["anthropic_version"], BEDROCK_VERSION);
    }

    #[test]
    fn bedrock_prepare_request_streaming() {
        let backend = BedrockBackend {
            auth: BedrockAuth::BearerToken("test".into()),
            region: "us-east-1".into(),
            base_url: "https://bedrock-runtime.us-east-1.amazonaws.com".into(),
        };

        let body = serde_json::json!({
            "model": "anthropic.claude-3-sonnet-20240229-v1:0",
            "max_tokens": 1024,
            "stream": true,
            "messages": [{"role": "user", "content": "Hello"}]
        });

        let req = BackendRequest {
            method: reqwest::Method::POST,
            path_segments: vec!["v1".into(), "messages".into()],
            query_params: vec![],
            headers: HeaderMap::new(),
            body: Some(body),
        };

        let prepared = backend.prepare_request(req).unwrap();

        assert_eq!(
            prepared.path_segments,
            vec!["model", "anthropic.claude-3-sonnet-20240229-v1:0", "invoke-with-response-stream"]
        );

        let body = prepared.body.unwrap();
        assert!(body.get("stream").is_none()); // stream removed from body
    }

    #[test]
    fn bedrock_rejects_batches() {
        let backend = BedrockBackend {
            auth: BedrockAuth::BearerToken("test".into()),
            region: "us-east-1".into(),
            base_url: "https://bedrock-runtime.us-east-1.amazonaws.com".into(),
        };

        let req = BackendRequest {
            method: reqwest::Method::POST,
            path_segments: vec!["v1".into(), "messages".into(), "batches".into()],
            query_params: vec![],
            headers: HeaderMap::new(),
            body: Some(serde_json::json!({})),
        };

        let err = backend.prepare_request(req).unwrap_err();
        assert!(err.to_string().contains("Batch API"));
    }

    #[test]
    fn sigv4_produces_valid_header() {
        let creds = AwsCredentials {
            access_key_id: "AKIAIOSFODNN7EXAMPLE".into(),
            secret_access_key: "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY".into(),
            session_token: None,
        };

        let url = url::Url::parse("https://bedrock-runtime.us-east-1.amazonaws.com/model/test/invoke").unwrap();
        let mut headers = HeaderMap::new();
        headers.insert("host", HeaderValue::from_static("bedrock-runtime.us-east-1.amazonaws.com"));
        headers.insert("x-amz-date", HeaderValue::from_static("20230101T120000Z"));
        headers.insert("content-type", HeaderValue::from_static("application/json"));

        let auth = sign_v4(&SigV4Params {
            method: "POST",
            url: &url,
            headers: &headers,
            body: b"{}",
            credentials: &creds,
            region: "us-east-1",
            service: "bedrock",
            amz_date: "20230101T120000Z",
            date_stamp: "20230101",
        });

        assert!(auth.starts_with("AWS4-HMAC-SHA256 Credential=AKIAIOSFODNN7EXAMPLE/20230101/us-east-1/bedrock/aws4_request"));
        assert!(auth.contains("SignedHeaders="));
        assert!(auth.contains("Signature="));
    }
}
