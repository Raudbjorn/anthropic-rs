//! Google Vertex AI backend.
//!
//! Sends requests to the Vertex AI API with Google OAuth2 bearer token
//! authentication and Vertex-specific URL rewriting.
//!
//! # Example
//!
//! ```no_run
//! use anthropic_rs::{Anthropic, backends::VertexBackend};
//!
//! # fn example() -> anthropic_rs::Result<()> {
//! let backend = VertexBackend::builder()
//!     .access_token("ya29.a0AfH6SM...")
//!     .region("us-central1")
//!     .project("my-gcp-project")
//!     .build()?;
//!
//! let client = Anthropic::builder().backend(backend).build()?;
//! # Ok(())
//! # }
//! ```

use reqwest::header::HeaderValue;

use crate::error::{AnthropicError, Result};

use super::{Backend, BackendRequest};

const VERTEX_VERSION: &str = "vertex-2023-10-16";

/// Backend for Google Vertex AI.
///
/// Transforms Anthropic API requests into Vertex AI format:
/// - Rewrites URL: `/v1/messages` → `/v1/projects/{project}/locations/{region}/publishers/anthropic/models/{model}:rawPredict`
/// - Adds `anthropic_version` to request body
/// - Uses Google OAuth2 bearer token for authorization
#[derive(Debug, Clone)]
pub struct VertexBackend {
    access_token: String,
    region: String,
    project: String,
    base_url: String,
}

impl VertexBackend {
    /// The GCP region this backend targets.
    pub fn region(&self) -> &str {
        &self.region
    }

    /// The GCP project ID.
    pub fn project(&self) -> &str {
        &self.project
    }
}

impl VertexBackend {
    pub fn builder() -> VertexBuilder {
        VertexBuilder::new()
    }

    /// Create from environment variables.
    ///
    /// Reads `ANTHROPIC_VERTEX_ACCESS_TOKEN`, `CLOUD_ML_REGION`,
    /// and `ANTHROPIC_VERTEX_PROJECT_ID`.
    pub fn from_env() -> Result<Self> {
        Self::builder().from_env().build()
    }
}

impl Backend for VertexBackend {
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
        let mut is_count_tokens = false;

        match service.as_str() {
            "messages" => {
                if req.path_segments.len() > 2 {
                    match req.path_segments[2].as_str() {
                        "batches" => {
                            return Err(AnthropicError::Config(
                                "Batch API is not supported for Vertex AI".into(),
                            ))
                        }
                        "count_tokens" => is_count_tokens = true,
                        _ => {}
                    }
                }
            }
            "complete" => {}
            other => {
                return Err(AnthropicError::Config(format!(
                    "Service is not supported for Vertex AI: {other}"
                )))
            }
        }

        // Transform body
        let body = req.body.as_mut().ok_or_else(|| {
            AnthropicError::InvalidData("Vertex request has no body".into())
        })?;
        let obj = body.as_object_mut().ok_or_else(|| {
            AnthropicError::InvalidData("Vertex request body is not an object".into())
        })?;

        // Add anthropic_version to body
        obj.insert(
            "anthropic_version".to_owned(),
            serde_json::Value::String(VERTEX_VERSION.to_owned()),
        );

        // Build endpoint path
        let endpoint = if is_count_tokens {
            "count-tokens:rawPredict".to_owned()
        } else {
            // Extract model from body
            let model_id = obj
                .remove("model")
                .and_then(|v| v.as_str().map(String::from))
                .ok_or_else(|| {
                    AnthropicError::InvalidData("No model found in request body".into())
                })?;

            let is_stream = obj.get("stream").and_then(|v| v.as_bool()).unwrap_or(false);
            let specifier = if is_stream {
                "streamRawPredict"
            } else {
                "rawPredict"
            };
            format!("{model_id}:{specifier}")
        };

        // Rewrite path: v1/projects/{project}/locations/{region}/publishers/anthropic/models/{endpoint}
        req.path_segments = vec![
            "v1".into(),
            "projects".into(),
            self.project.clone(),
            "locations".into(),
            self.region.clone(),
            "publishers".into(),
            "anthropic".into(),
            "models".into(),
            endpoint,
        ];

        // Set content-type
        req.headers
            .insert("content-type", HeaderValue::from_static("application/json"));

        Ok(req)
    }

    fn authorize_request(
        &self,
        mut req: BackendRequest,
        _resolved_url: &url::Url,
    ) -> Result<BackendRequest> {
        req.headers.insert(
            "authorization",
            HeaderValue::from_str(&format!("Bearer {}", self.access_token))
                .map_err(|e| AnthropicError::Config(format!("invalid access token: {e}")))?,
        );
        Ok(req)
    }
}

/// Builder for [`VertexBackend`].
pub struct VertexBuilder {
    access_token: Option<String>,
    region: Option<String>,
    project: Option<String>,
}

impl VertexBuilder {
    fn new() -> Self {
        Self {
            access_token: None,
            region: None,
            project: None,
        }
    }

    /// Load configuration from environment variables.
    pub fn from_env(mut self) -> Self {
        if let Ok(token) = std::env::var("ANTHROPIC_VERTEX_ACCESS_TOKEN") {
            self.access_token = Some(token);
        }
        if let Ok(region) = std::env::var("CLOUD_ML_REGION") {
            self.region = Some(region);
        }
        if let Ok(project) = std::env::var("ANTHROPIC_VERTEX_PROJECT_ID") {
            self.project = Some(project);
        }
        self
    }

    /// Set the Google OAuth2 access token.
    ///
    /// For dynamic token refresh, rebuild the backend periodically or use
    /// a token management wrapper.
    pub fn access_token(mut self, token: impl Into<String>) -> Self {
        self.access_token = Some(token.into());
        self
    }

    pub fn region(mut self, region: impl Into<String>) -> Self {
        self.region = Some(region.into());
        self
    }

    pub fn project(mut self, project: impl Into<String>) -> Self {
        self.project = Some(project.into());
        self
    }

    pub fn build(self) -> Result<VertexBackend> {
        let access_token = self.access_token.ok_or_else(|| {
            AnthropicError::Config(
                "Google access token must be provided. Set ANTHROPIC_VERTEX_ACCESS_TOKEN or use .access_token()."
                    .into(),
            )
        })?;
        let region = self.region.ok_or_else(|| {
            AnthropicError::Config(
                "Region must be set. Use CLOUD_ML_REGION env var or .region().".into(),
            )
        })?;
        let project = self.project.ok_or_else(|| {
            AnthropicError::Config(
                "Project must be set. Use ANTHROPIC_VERTEX_PROJECT_ID env var or .project()."
                    .into(),
            )
        })?;

        let base_url = if region == "global" {
            "https://aiplatform.googleapis.com".to_owned()
        } else {
            format!("https://{region}-aiplatform.googleapis.com")
        };

        Ok(VertexBackend {
            access_token,
            region,
            project,
            base_url,
        })
    }
}

impl Default for VertexBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use reqwest::header::HeaderMap;

    use super::*;

    #[test]
    fn vertex_prepare_request_transforms_path() {
        let backend = VertexBackend {
            access_token: "test-token".into(),
            region: "us-central1".into(),
            project: "my-project".into(),
            base_url: "https://us-central1-aiplatform.googleapis.com".into(),
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
            vec![
                "v1",
                "projects",
                "my-project",
                "locations",
                "us-central1",
                "publishers",
                "anthropic",
                "models",
                "claude-sonnet-4-6:rawPredict"
            ]
        );

        let body = prepared.body.unwrap();
        assert!(body.get("model").is_none());
        assert_eq!(body["anthropic_version"], VERTEX_VERSION);
    }

    #[test]
    fn vertex_prepare_request_streaming() {
        let backend = VertexBackend {
            access_token: "test-token".into(),
            region: "us-central1".into(),
            project: "my-project".into(),
            base_url: "https://us-central1-aiplatform.googleapis.com".into(),
        };

        let body = serde_json::json!({
            "model": "claude-sonnet-4-6",
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

        assert!(prepared.path_segments.last().unwrap().ends_with(":streamRawPredict"));
    }

    #[test]
    fn vertex_count_tokens() {
        let backend = VertexBackend {
            access_token: "test-token".into(),
            region: "us-central1".into(),
            project: "my-project".into(),
            base_url: "https://us-central1-aiplatform.googleapis.com".into(),
        };

        let body = serde_json::json!({
            "model": "claude-sonnet-4-6",
            "max_tokens": 1024,
            "messages": [{"role": "user", "content": "Hello"}]
        });

        let req = BackendRequest {
            method: reqwest::Method::POST,
            path_segments: vec!["v1".into(), "messages".into(), "count_tokens".into()],
            query_params: vec![],
            headers: HeaderMap::new(),
            body: Some(body),
        };

        let prepared = backend.prepare_request(req).unwrap();

        assert!(prepared.path_segments.last().unwrap().contains("count-tokens:rawPredict"));
    }

    #[test]
    fn vertex_rejects_batches() {
        let backend = VertexBackend {
            access_token: "test-token".into(),
            region: "us-central1".into(),
            project: "my-project".into(),
            base_url: "https://us-central1-aiplatform.googleapis.com".into(),
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
    fn vertex_base_url_regional() {
        let backend = VertexBackend::builder()
            .access_token("t")
            .region("us-central1")
            .project("p")
            .build()
            .unwrap();
        assert_eq!(
            backend.base_url(),
            "https://us-central1-aiplatform.googleapis.com"
        );
    }

    #[test]
    fn vertex_base_url_global() {
        let backend = VertexBackend::builder()
            .access_token("t")
            .region("global")
            .project("p")
            .build()
            .unwrap();
        assert_eq!(
            backend.base_url(),
            "https://aiplatform.googleapis.com"
        );
    }
}
