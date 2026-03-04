//! OAuth 2.0 Authorization Code + PKCE login for the Anthropic API.
//!
//! This module provides:
//! - [`OAuthFlow`] — orchestrates the full OAuth login/refresh/logout cycle
//! - [`OAuthBackend`] — implements [`Backend`] for use with the Anthropic client
//! - [`TokenStorage`] — trait for persisting tokens
//! - [`CallbackServer`] — localhost server for capturing OAuth redirects
//!
//! # Quick Start
//!
//! ```no_run
//! use anthropic_rs::oauth::{OAuthBackend, FileTokenStorage};
//! use anthropic_rs::Anthropic;
//!
//! # fn example() -> anthropic_rs::Result<()> {
//! let storage = FileTokenStorage::default_path()?;
//! let backend = OAuthBackend::new(storage);
//! let client = Anthropic::builder().backend(backend).build()?;
//! # Ok(())
//! # }
//! ```

pub mod callback_server;
pub mod config;
pub mod error;
pub mod flow;
pub mod pkce;
pub mod storage;
pub mod token;

pub use callback_server::{CallbackResult, CallbackServer};
pub use config::OAuthConfig;
pub use error::OAuthError;
pub use flow::{OAuthFlow, OAuthFlowState};
pub use pkce::Pkce;
pub use storage::{FileTokenStorage, MemoryTokenStorage, TokenStorage};
pub use token::TokenInfo;

use std::sync::Arc;

use reqwest::header::HeaderValue;
use tracing::debug;

use crate::backends::{Backend, BackendRequest};
use crate::error::{AnthropicError, Result};

const ANTHROPIC_VERSION: &str = "2023-06-01";
const DEFAULT_BETAS: &[&str] = &[
    "interleaved-thinking-2025-05-14",
    "code-execution-2025-05-22",
    "oauth-2025-04-20",
];

/// OAuth-based backend for the Anthropic API.
///
/// Uses stored OAuth tokens for authentication instead of API keys.
/// Tokens are refreshed automatically when they're within 5 minutes of expiry.
pub struct OAuthBackend {
    flow: Arc<tokio::sync::RwLock<OAuthFlow<Box<dyn TokenStorage>>>>,
    /// Cached access token for sync `authorize_request`.
    cached_token: Arc<std::sync::RwLock<Option<String>>>,
    base_url: String,
    betas: Vec<String>,
}

impl OAuthBackend {
    /// Create an OAuthBackend with the given storage.
    pub fn new(storage: impl TokenStorage + 'static) -> Self {
        Self::with_config(storage, OAuthConfig::default())
    }

    /// Create an OAuthBackend with custom config.
    pub fn with_config(storage: impl TokenStorage + 'static, config: OAuthConfig) -> Self {
        let boxed: Box<dyn TokenStorage> = Box::new(storage);
        let flow = OAuthFlow::with_config(boxed, config);
        Self {
            flow: Arc::new(tokio::sync::RwLock::new(flow)),
            cached_token: Arc::new(std::sync::RwLock::new(None)),
            base_url: "https://api.anthropic.com".to_string(),
            betas: DEFAULT_BETAS.iter().map(|s| s.to_string()).collect(),
        }
    }

    /// Set a custom base URL.
    pub fn with_base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = url.into();
        self
    }

    /// Override the beta features.
    pub fn with_betas(mut self, betas: Vec<String>) -> Self {
        self.betas = betas;
        self
    }

    /// Get a reference to the inner OAuthFlow for manual operations
    /// (login, logout, status).
    pub fn flow(&self) -> &Arc<tokio::sync::RwLock<OAuthFlow<Box<dyn TokenStorage>>>> {
        &self.flow
    }
}

impl std::fmt::Debug for OAuthBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OAuthBackend")
            .field("base_url", &self.base_url)
            .finish_non_exhaustive()
    }
}

impl Backend for OAuthBackend {
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
        let guard = self.cached_token.read().map_err(|e| {
            AnthropicError::Config(format!("lock poisoned: {e}"))
        })?;
        if let Some(ref token) = *guard {
            req.headers.insert(
                "authorization",
                HeaderValue::from_str(&format!("Bearer {token}"))
                    .map_err(|e| AnthropicError::Config(format!("invalid auth token: {e}")))?,
            );
        }
        Ok(req)
    }

    fn pre_request(&self) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + '_>> {
        Box::pin(async move {
            let flow = self.flow.read().await;
            match flow.storage().load() {
                Ok(Some(token)) if token.needs_refresh() => {
                    debug!("OAuth token needs refresh");
                    match flow.refresh_token().await {
                        Ok(new_token) => {
                            let mut guard = self.cached_token.write().map_err(|e| {
                                AnthropicError::Config(format!("lock poisoned: {e}"))
                            })?;
                            *guard = Some(new_token.access_token);
                        }
                        Err(e) => {
                            return Err(AnthropicError::OAuth(format!("token refresh failed: {e}")));
                        }
                    }
                }
                Ok(Some(token)) => {
                    let mut guard = self.cached_token.write().map_err(|e| {
                        AnthropicError::Config(format!("lock poisoned: {e}"))
                    })?;
                    *guard = Some(token.access_token);
                }
                Ok(None) => {
                    return Err(AnthropicError::OAuth("not authenticated; perform OAuth login first".into()));
                }
                Err(e) => {
                    return Err(AnthropicError::OAuth(format!("failed to load token: {e}")));
                }
            }
            Ok(())
        })
    }
}
