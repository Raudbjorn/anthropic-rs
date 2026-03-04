//! OAuth-specific error types.

use std::fmt;

/// All errors that can occur during OAuth operations.
#[derive(Debug)]
pub enum OAuthError {
    /// Generic OAuth error.
    OAuth(String),
    /// No valid token exists.
    NotAuthenticated,
    /// Token refresh failed.
    RefreshFailed(String),
    /// State parameter mismatch (CSRF protection).
    InvalidState {
        expected: String,
        actual: String,
    },
    /// Callback server error.
    CallbackServer(String),
    /// Token storage error.
    Storage(String),
    /// HTTP request error.
    Http(reqwest::Error),
    /// IO error.
    Io(std::io::Error),
    /// JSON serialization error.
    Json(serde_json::Error),
}

impl fmt::Display for OAuthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OAuth(msg) => write!(f, "OAuth error: {msg}"),
            Self::NotAuthenticated => write!(f, "not authenticated"),
            Self::RefreshFailed(msg) => write!(f, "token refresh failed: {msg}"),
            Self::InvalidState { expected, actual } => {
                write!(f, "state mismatch: expected {expected}, got {actual}")
            }
            Self::CallbackServer(msg) => write!(f, "callback server error: {msg}"),
            Self::Storage(msg) => write!(f, "storage error: {msg}"),
            Self::Http(e) => write!(f, "HTTP error: {e}"),
            Self::Io(e) => write!(f, "IO error: {e}"),
            Self::Json(e) => write!(f, "JSON error: {e}"),
        }
    }
}

impl std::error::Error for OAuthError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Http(e) => Some(e),
            Self::Io(e) => Some(e),
            Self::Json(e) => Some(e),
            _ => None,
        }
    }
}

impl From<reqwest::Error> for OAuthError {
    fn from(e: reqwest::Error) -> Self {
        Self::Http(e)
    }
}

impl From<std::io::Error> for OAuthError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<serde_json::Error> for OAuthError {
    fn from(e: serde_json::Error) -> Self {
        Self::Json(e)
    }
}

impl From<url::ParseError> for OAuthError {
    fn from(e: url::ParseError) -> Self {
        Self::OAuth(format!("invalid URL: {e}"))
    }
}

impl From<OAuthError> for crate::error::AnthropicError {
    fn from(e: OAuthError) -> Self {
        crate::error::AnthropicError::OAuth(e.to_string())
    }
}

/// Convenience Result alias for OAuth operations.
pub type Result<T> = std::result::Result<T, OAuthError>;
