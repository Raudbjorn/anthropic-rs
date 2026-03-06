use reqwest::header::HeaderMap;
use std::fmt;

/// Categorized HTTP error kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpErrorKind {
    BadRequest,
    Unauthorized,
    Billing,
    PermissionDenied,
    NotFound,
    UnprocessableEntity,
    RateLimited,
    InternalServer,
    GatewayTimeout,
    Overloaded,
    UnexpectedStatus,
}

impl fmt::Display for HttpErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BadRequest => write!(f, "bad request"),
            Self::Unauthorized => write!(f, "unauthorized"),
            Self::Billing => write!(f, "billing error"),
            Self::PermissionDenied => write!(f, "permission denied"),
            Self::NotFound => write!(f, "not found"),
            Self::UnprocessableEntity => write!(f, "unprocessable entity"),
            Self::RateLimited => write!(f, "rate limited"),
            Self::InternalServer => write!(f, "internal server error"),
            Self::GatewayTimeout => write!(f, "gateway timeout"),
            Self::Overloaded => write!(f, "overloaded"),
            Self::UnexpectedStatus => write!(f, "unexpected status"),
        }
    }
}

/// Details of an HTTP API error.
#[derive(Debug)]
pub struct HttpErrorDetails {
    pub kind: HttpErrorKind,
    pub message: String,
    pub status: u16,
    pub headers: HeaderMap,
    pub body: String,
}

/// All errors that can occur when using the Anthropic SDK.
#[derive(Debug, thiserror::Error)]
pub enum AnthropicError {
    /// HTTP API error (4xx/5xx responses).
    #[error("{} ({}): {}", .0.kind, .0.status, .0.message)]
    Api(Box<HttpErrorDetails>),

    /// SSE stream parsing error.
    #[error("SSE error: {0}")]
    Sse(String),

    /// IO error (network, file, etc.).
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// HTTP client error.
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    /// JSON serialization/deserialization error.
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Invalid data in response.
    #[error("invalid data: {0}")]
    InvalidData(String),

    /// Configuration error (missing API key, invalid base URL, etc.).
    #[error("configuration error: {0}")]
    Config(String),

    /// OAuth authentication error.
    #[cfg(feature = "oauth")]
    #[error("OAuth error: {0}")]
    OAuth(String),

    /// Realtime WebSocket API error.
    #[cfg(feature = "realtime")]
    #[error("Realtime error: {0}")]
    Realtime(crate::realtime::error::RealtimeErrorKind),
}

impl AnthropicError {
    /// Whether this error is retryable (429, 5xx, IO, overloaded).
    pub fn is_retryable(&self) -> bool {
        match self {
            Self::Api(details) => matches!(
                details.kind,
                HttpErrorKind::RateLimited
                    | HttpErrorKind::InternalServer
                    | HttpErrorKind::Overloaded
            ) || matches!(details.status, 408 | 409),
            Self::Io(_) | Self::Http(_) => true,
            _ => false,
        }
    }

    /// Build from an HTTP response status + body.
    /// Build an error from an HTTP response status, headers, and body text.
    pub fn from_status(status: u16, headers: HeaderMap, body: String) -> Self {
        let message = extract_error_message(&body).unwrap_or_else(|| body.clone());
        let kind = match status {
            400 => HttpErrorKind::BadRequest,
            401 => HttpErrorKind::Unauthorized,
            402 => HttpErrorKind::Billing,
            403 => HttpErrorKind::PermissionDenied,
            404 => HttpErrorKind::NotFound,
            422 => HttpErrorKind::UnprocessableEntity,
            429 => HttpErrorKind::RateLimited,
            504 => HttpErrorKind::GatewayTimeout,
            529 => HttpErrorKind::Overloaded,
            500..=599 => HttpErrorKind::InternalServer,
            _ => HttpErrorKind::UnexpectedStatus,
        };
        Self::Api(Box::new(HttpErrorDetails {
            kind,
            message,
            status,
            headers,
            body,
        }))
    }

    /// Extract status code if this is an HTTP error.
    pub fn status(&self) -> Option<u16> {
        match self {
            Self::Api(d) => Some(d.status),
            _ => None,
        }
    }

    /// Extract headers if this is an HTTP error.
    pub fn headers(&self) -> Option<&HeaderMap> {
        match self {
            Self::Api(d) => Some(&d.headers),
            _ => None,
        }
    }

    /// Extract the error kind if this is an HTTP error.
    pub fn kind(&self) -> Option<HttpErrorKind> {
        match self {
            Self::Api(d) => Some(d.kind),
            _ => None,
        }
    }

    /// Check if this is a specific HTTP error kind.
    pub fn is_kind(&self, kind: HttpErrorKind) -> bool {
        self.kind() == Some(kind)
    }
}

fn extract_error_message(body: &str) -> Option<String> {
    let v: serde_json::Value = serde_json::from_str(body).ok()?;
    v.get("error")
        .and_then(|e| e.get("message"))
        .and_then(|m| m.as_str())
        .map(String::from)
}

/// API error object returned in JSON error responses.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct ErrorResponse {
    #[serde(rename = "type")]
    pub error_type: String,
    pub error: ApiErrorObject,
}

/// The nested error object.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct ApiErrorObject {
    #[serde(rename = "type")]
    pub error_type: String,
    pub message: String,
    #[serde(default)]
    pub param: Option<String>,
}

/// Convenience Result alias.
pub type Result<T> = std::result::Result<T, AnthropicError>;

impl fmt::Display for ErrorResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.error.error_type, self.error.message)
    }
}
