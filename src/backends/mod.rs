//! Cloud provider backends for the Anthropic SDK.
//!
//! Backends handle URL construction, request transformation, and authorization
//! for different API providers:
//!
//! - [`AnthropicBackend`] — Direct Anthropic API (default)
//! - [`BedrockBackend`] — AWS Bedrock Runtime (feature: `bedrock`)
//! - [`VertexBackend`] — Google Vertex AI (feature: `vertex`)
//! - [`FoundryBackend`] — Azure AI Foundry (feature: `foundry`)

mod anthropic_backend;
#[cfg(feature = "bedrock")]
pub mod bedrock;
#[cfg(feature = "foundry")]
mod foundry;
#[cfg(feature = "vertex")]
mod vertex;

pub use anthropic_backend::{AnthropicBackend, AnthropicBackendBuilder};
#[cfg(feature = "bedrock")]
pub use bedrock::{AwsCredentials, BedrockBackend, BedrockBuilder};
#[cfg(feature = "foundry")]
pub use foundry::{FoundryBackend, FoundryBuilder};
#[cfg(feature = "vertex")]
pub use vertex::{VertexBackend, VertexBuilder};

use crate::error::{AnthropicError, Result};
use reqwest::header::HeaderMap;

/// A request being prepared for a backend.
///
/// The client populates this from the API method call, then the backend
/// transforms it (rewriting paths, bodies, headers) before it's sent.
#[derive(Debug, Clone)]
pub struct BackendRequest {
    /// HTTP method (POST, GET, DELETE, etc.).
    pub method: reqwest::Method,
    /// URL path segments (e.g., `["v1", "messages"]`).
    pub path_segments: Vec<String>,
    /// Query parameters.
    pub query_params: Vec<(String, String)>,
    /// HTTP headers.
    pub headers: HeaderMap,
    /// JSON request body (if any).
    pub body: Option<serde_json::Value>,
}

impl BackendRequest {
    /// Resolve the full URL from the base URL and path segments.
    pub fn resolve_url(&self, base_url: &str) -> Result<url::Url> {
        let base = base_url.trim_end_matches('/');
        let path = self.path_segments.join("/");
        let url_str = format!("{base}/{path}");
        let mut url = url::Url::parse(&url_str)
            .map_err(|e| AnthropicError::Config(format!("invalid URL '{url_str}': {e}")))?;
        for (k, v) in &self.query_params {
            url.query_pairs_mut().append_pair(k, v);
        }
        Ok(url)
    }
}

/// Trait for cloud provider backends.
///
/// Backends control how requests are constructed, authorized, and optionally
/// how streaming responses are transformed. The client calls methods in order:
///
/// 1. [`prepare_request`](Backend::prepare_request) — rewrite path, body, add version headers
/// 2. URL resolution from [`base_url`](Backend::base_url) + transformed path segments
/// 3. [`authorize_request`](Backend::authorize_request) — add auth headers / sign request
/// 4. Send HTTP request
/// 5. Optionally transform streaming response via [`stream_transformer`](Backend::stream_transformer)
pub trait Backend: Send + Sync {
    /// The base URL for this backend's API endpoint.
    fn base_url(&self) -> &str;

    /// Prepare the request: transform URL path, body, and add version headers.
    ///
    /// Called before URL resolution and authorization. Implementations should
    /// rewrite `path_segments`, modify the JSON body (e.g., move `model` field
    /// to URL for Bedrock), and add version headers.
    fn prepare_request(&self, req: BackendRequest) -> Result<BackendRequest> {
        Ok(req)
    }

    /// Authorize the request: add auth headers or sign the request.
    ///
    /// Called after `prepare_request` and URL resolution. The fully resolved
    /// URL is provided for backends that need it (e.g., AWS SigV4 signing).
    fn authorize_request(
        &self,
        req: BackendRequest,
        resolved_url: &url::Url,
    ) -> Result<BackendRequest>;

    /// Create a stream transformer for converting this backend's streaming
    /// response format to standard SSE.
    ///
    /// Returns `None` if no transformation is needed (the default).
    /// Only Bedrock needs this (AWS event-stream binary → SSE text).
    fn stream_transformer(&self) -> Option<Box<dyn StreamTransformer>> {
        None
    }

    /// Async hook called before each request.
    ///
    /// Used by backends that need pre-flight operations (e.g., token refresh).
    /// The default is a no-op.
    fn pre_request(&self) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + '_>> {
        Box::pin(async { Ok(()) })
    }
}

/// Transforms streaming response bytes from a backend-specific format to SSE.
///
/// Implementations are stateful to handle partial frame buffering.
pub trait StreamTransformer: Send {
    /// Feed raw bytes from the backend and append transformed SSE bytes to `output`.
    fn transform(&mut self, data: &[u8], output: &mut Vec<u8>) -> Result<()>;
}
