//! Azure AI Foundry backend.
//!
//! Sends requests to the Azure AI Foundry API with URL prefix rewriting
//! and API key or bearer token authentication.
//!
//! # Example
//!
//! ```no_run
//! use anthropic_rs::{Anthropic, backends::FoundryBackend};
//!
//! # fn example() -> anthropic_rs::Result<()> {
//! let backend = FoundryBackend::builder()
//!     .api_key("my-foundry-api-key")
//!     .resource("my-resource")
//!     .build()?;
//!
//! let client = Anthropic::builder().backend(backend).build()?;
//! # Ok(())
//! # }
//! ```

use reqwest::header::HeaderValue;

use crate::error::{AnthropicError, Result};

use super::{Backend, BackendRequest};

const ANTHROPIC_VERSION: &str = "2023-06-01";

/// Authentication method for Foundry.
#[derive(Debug, Clone)]
enum FoundryAuth {
    ApiKey(String),
    BearerToken(String),
}

/// Backend for Azure AI Foundry.
///
/// Transforms Anthropic API requests into Foundry-compatible format:
/// - Prefixes URL path: `/v1/messages` → `/anthropic/v1/messages`
/// - Adds `anthropic-version` header
/// - Uses API key or bearer token for authorization
#[derive(Debug, Clone)]
pub struct FoundryBackend {
    auth: FoundryAuth,
    base_url: String,
}

impl FoundryBackend {
    pub fn builder() -> FoundryBuilder {
        FoundryBuilder::new()
    }

    /// Create from environment variables.
    ///
    /// Reads `ANTHROPIC_FOUNDRY_API_KEY`, and one of `ANTHROPIC_FOUNDRY_RESOURCE`
    /// or `ANTHROPIC_FOUNDRY_BASE_URL`.
    pub fn from_env() -> Result<Self> {
        Self::builder().from_env().build()
    }
}

impl Backend for FoundryBackend {
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
                if req.path_segments.len() > 2 && req.path_segments[2] == "batches" {
                    return Err(AnthropicError::Config(
                        "Batch API is not supported for Foundry".into(),
                    ));
                }
            }
            "skills" | "files" => {}
            other => {
                return Err(AnthropicError::Config(format!(
                    "Service is not supported for Foundry: {other}"
                )))
            }
        }

        // Prefix path with "anthropic": /v1/messages → /anthropic/v1/messages
        let mut new_segments = vec!["anthropic".to_owned()];
        new_segments.extend(req.path_segments);
        req.path_segments = new_segments;

        // Add version header
        req.headers.insert(
            "anthropic-version",
            HeaderValue::from_static(ANTHROPIC_VERSION),
        );
        req.headers
            .insert("content-type", HeaderValue::from_static("application/json"));

        Ok(req)
    }

    fn authorize_request(
        &self,
        mut req: BackendRequest,
        _resolved_url: &url::Url,
    ) -> Result<BackendRequest> {
        match &self.auth {
            FoundryAuth::ApiKey(key) => {
                req.headers.insert(
                    "x-api-key",
                    HeaderValue::from_str(key)
                        .map_err(|e| AnthropicError::Config(format!("invalid API key: {e}")))?,
                );
            }
            FoundryAuth::BearerToken(token) => {
                req.headers.insert(
                    "authorization",
                    HeaderValue::from_str(&format!("Bearer {token}"))
                        .map_err(|e| AnthropicError::Config(format!("invalid token: {e}")))?,
                );
            }
        }
        Ok(req)
    }
}

/// Builder for [`FoundryBackend`].
pub struct FoundryBuilder {
    api_key: Option<String>,
    bearer_token: Option<String>,
    resource: Option<String>,
    base_url: Option<String>,
}

impl FoundryBuilder {
    fn new() -> Self {
        Self {
            api_key: None,
            bearer_token: None,
            resource: None,
            base_url: None,
        }
    }

    /// Load configuration from environment variables.
    pub fn from_env(mut self) -> Self {
        if let Ok(key) = std::env::var("ANTHROPIC_FOUNDRY_API_KEY") {
            self.api_key = Some(key);
        }
        self.resource = std::env::var("ANTHROPIC_FOUNDRY_RESOURCE").ok();
        self.base_url = std::env::var("ANTHROPIC_FOUNDRY_BASE_URL").ok();
        self
    }

    pub fn api_key(mut self, key: impl Into<String>) -> Self {
        self.api_key = Some(key.into());
        self.bearer_token = None;
        self
    }

    pub fn bearer_token(mut self, token: impl Into<String>) -> Self {
        self.bearer_token = Some(token.into());
        self.api_key = None;
        self
    }

    /// Set the Azure resource name.
    ///
    /// The base URL will be `https://{resource}.services.ai.azure.com`.
    /// Mutually exclusive with `base_url`.
    pub fn resource(mut self, resource: impl Into<String>) -> Self {
        self.resource = Some(resource.into());
        self.base_url = None;
        self
    }

    /// Set a custom base URL directly.
    ///
    /// Mutually exclusive with `resource`.
    pub fn base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = Some(url.into());
        self.resource = None;
        self
    }

    pub fn build(self) -> Result<FoundryBackend> {
        let auth = if let Some(key) = self.api_key {
            FoundryAuth::ApiKey(key)
        } else if let Some(token) = self.bearer_token {
            FoundryAuth::BearerToken(token)
        } else {
            return Err(AnthropicError::Config(
                "API key or bearer token must be provided for Foundry. \
                 Set ANTHROPIC_FOUNDRY_API_KEY or use .api_key() / .bearer_token()."
                    .into(),
            ));
        };

        let base_url = if let Some(url) = self.base_url {
            url
        } else if let Some(resource) = self.resource {
            format!("https://{resource}.services.ai.azure.com")
        } else {
            return Err(AnthropicError::Config(
                "Resource or base URL must be set for Foundry. \
                 Set ANTHROPIC_FOUNDRY_RESOURCE or ANTHROPIC_FOUNDRY_BASE_URL."
                    .into(),
            ));
        };

        Ok(FoundryBackend { auth, base_url })
    }
}

impl Default for FoundryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use reqwest::header::HeaderMap;

    use super::*;

    #[test]
    fn foundry_prepare_request_prefixes_path() {
        let backend = FoundryBackend {
            auth: FoundryAuth::ApiKey("test-key".into()),
            base_url: "https://my-resource.services.ai.azure.com".into(),
        };

        let body = serde_json::json!({
            "model": "claude-sonnet-4-6",
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

        assert_eq!(
            prepared.path_segments,
            vec!["anthropic", "v1", "messages"]
        );
        assert_eq!(
            prepared.headers.get("anthropic-version").unwrap(),
            ANTHROPIC_VERSION
        );
    }

    #[test]
    fn foundry_authorize_with_api_key() {
        let backend = FoundryBackend {
            auth: FoundryAuth::ApiKey("test-key".into()),
            base_url: "https://example.com".into(),
        };

        let req = BackendRequest {
            method: reqwest::Method::POST,
            path_segments: vec![],
            query_params: vec![],
            headers: HeaderMap::new(),
            body: None,
        };

        let url = url::Url::parse("https://example.com/anthropic/v1/messages").unwrap();
        let authed = backend.authorize_request(req, &url).unwrap();

        assert_eq!(authed.headers.get("x-api-key").unwrap(), "test-key");
    }

    #[test]
    fn foundry_authorize_with_bearer() {
        let backend = FoundryBackend {
            auth: FoundryAuth::BearerToken("my-token".into()),
            base_url: "https://example.com".into(),
        };

        let req = BackendRequest {
            method: reqwest::Method::POST,
            path_segments: vec![],
            query_params: vec![],
            headers: HeaderMap::new(),
            body: None,
        };

        let url = url::Url::parse("https://example.com/anthropic/v1/messages").unwrap();
        let authed = backend.authorize_request(req, &url).unwrap();

        assert_eq!(
            authed.headers.get("authorization").unwrap(),
            "Bearer my-token"
        );
    }

    #[test]
    fn foundry_rejects_batches() {
        let backend = FoundryBackend {
            auth: FoundryAuth::ApiKey("test-key".into()),
            base_url: "https://example.com".into(),
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
    fn foundry_builder_resource_url() {
        let backend = FoundryBackend::builder()
            .api_key("key")
            .resource("my-resource")
            .build()
            .unwrap();

        assert_eq!(
            backend.base_url,
            "https://my-resource.services.ai.azure.com"
        );
    }

    #[test]
    fn foundry_builder_custom_url() {
        let backend = FoundryBackend::builder()
            .api_key("key")
            .base_url("https://custom.example.com")
            .build()
            .unwrap();

        assert_eq!(backend.base_url, "https://custom.example.com");
    }
}
