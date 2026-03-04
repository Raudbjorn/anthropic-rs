use reqwest::header::HeaderValue;

use crate::error::{AnthropicError, Result};

use super::{Backend, BackendRequest};

const DEFAULT_BASE_URL: &str = "https://api.anthropic.com";
const ANTHROPIC_VERSION: &str = "2023-06-01";
const DEFAULT_BETAS: &[&str] = &[
    "interleaved-thinking-2025-05-14",
    "code-execution-2025-05-22",
];

/// Backend for the direct Anthropic API.
///
/// This is the default backend used when constructing an `Anthropic` client
/// without specifying a custom backend. It sends requests directly to
/// `api.anthropic.com` with API key or bearer token authentication.
#[derive(Debug, Clone)]
pub struct AnthropicBackend {
    api_key: Option<String>,
    auth_token: Option<String>,
    base_url: String,
    betas: Vec<String>,
}

impl AnthropicBackend {
    pub fn builder() -> AnthropicBackendBuilder {
        AnthropicBackendBuilder::new()
    }

    pub fn from_env() -> Result<Self> {
        Self::builder().from_env().build()
    }

}

impl Backend for AnthropicBackend {
    fn base_url(&self) -> &str {
        &self.base_url
    }

    fn prepare_request(&self, mut req: BackendRequest) -> Result<BackendRequest> {
        req.headers.insert(
            "anthropic-version",
            HeaderValue::from_static(ANTHROPIC_VERSION),
        );
        if !self.betas.is_empty() {
            let beta_str = self.betas.join(",");
            req.headers.insert(
                "anthropic-beta",
                HeaderValue::from_str(&beta_str)
                    .map_err(|e| AnthropicError::Config(format!("invalid beta header: {e}")))?,
            );
        }
        req.headers
            .insert("content-type", HeaderValue::from_static("application/json"));
        Ok(req)
    }

    fn authorize_request(
        &self,
        mut req: BackendRequest,
        _resolved_url: &url::Url,
    ) -> Result<BackendRequest> {
        if let Some(ref key) = self.api_key {
            req.headers.insert(
                "x-api-key",
                HeaderValue::from_str(key)
                    .map_err(|e| AnthropicError::Config(format!("invalid API key: {e}")))?,
            );
        }
        if let Some(ref token) = self.auth_token {
            req.headers.insert(
                "authorization",
                HeaderValue::from_str(&format!("Bearer {token}"))
                    .map_err(|e| AnthropicError::Config(format!("invalid auth token: {e}")))?,
            );
        }
        Ok(req)
    }
}

/// Builder for [`AnthropicBackend`].
pub struct AnthropicBackendBuilder {
    api_key: Option<String>,
    auth_token: Option<String>,
    base_url: Option<String>,
    betas: Option<Vec<String>>,
}

impl AnthropicBackendBuilder {
    fn new() -> Self {
        Self {
            api_key: None,
            auth_token: None,
            base_url: None,
            betas: None,
        }
    }

    /// Load configuration from environment variables.
    ///
    /// Reads `ANTHROPIC_API_KEY`, `ANTHROPIC_AUTH_TOKEN`, and `ANTHROPIC_BASE_URL`.
    pub fn from_env(mut self) -> Self {
        if let Ok(key) = std::env::var("ANTHROPIC_API_KEY") {
            self.api_key = Some(key);
        }
        if let Ok(token) = std::env::var("ANTHROPIC_AUTH_TOKEN") {
            self.auth_token = Some(token);
        }
        if let Ok(url) = std::env::var("ANTHROPIC_BASE_URL") {
            self.base_url = Some(url);
        }
        self
    }

    pub fn api_key(mut self, key: impl Into<String>) -> Self {
        self.api_key = Some(key.into());
        self
    }

    pub fn auth_token(mut self, token: impl Into<String>) -> Self {
        self.auth_token = Some(token.into());
        self
    }

    pub fn base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = Some(url.into());
        self
    }

    /// Override the beta features sent in the `anthropic-beta` header.
    ///
    /// By default, `interleaved-thinking-2025-05-14` and
    /// `code-execution-2025-05-22` are enabled.
    pub fn betas(mut self, betas: Vec<String>) -> Self {
        self.betas = Some(betas);
        self
    }

    /// Add a single beta feature to the default set.
    pub fn beta(mut self, beta: impl Into<String>) -> Self {
        let betas = self.betas.get_or_insert_with(|| {
            DEFAULT_BETAS.iter().map(|s| s.to_string()).collect()
        });
        betas.push(beta.into());
        self
    }

    pub fn build(self) -> Result<AnthropicBackend> {
        if self.api_key.is_none() && self.auth_token.is_none() {
            return Err(AnthropicError::Config(
                "No API key or auth token provided. Set ANTHROPIC_API_KEY or ANTHROPIC_AUTH_TOKEN."
                    .into(),
            ));
        }
        let betas = self
            .betas
            .unwrap_or_else(|| DEFAULT_BETAS.iter().map(|s| s.to_string()).collect());

        Ok(AnthropicBackend {
            api_key: self.api_key,
            auth_token: self.auth_token,
            base_url: self
                .base_url
                .unwrap_or_else(|| DEFAULT_BASE_URL.to_owned()),
            betas,
        })
    }
}

impl Default for AnthropicBackendBuilder {
    fn default() -> Self {
        Self::new()
    }
}
