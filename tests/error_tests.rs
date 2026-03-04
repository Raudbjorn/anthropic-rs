use anthropic_rs::error::{AnthropicError, HttpErrorKind};
use reqwest::header::HeaderMap;

#[test]
fn error_from_status_400() {
    let err = AnthropicError::from_status(
        400,
        HeaderMap::new(),
        r#"{"error":{"type":"invalid_request_error","message":"bad param"}}"#.into(),
    );
    assert_eq!(err.status(), Some(400));
    assert!(err.is_kind(HttpErrorKind::BadRequest));
    assert!(!err.is_retryable());
    assert!(err.to_string().contains("bad param"));
}

#[test]
fn error_from_status_429() {
    let err = AnthropicError::from_status(429, HeaderMap::new(), "{}".into());
    assert_eq!(err.status(), Some(429));
    assert!(err.is_kind(HttpErrorKind::RateLimited));
    assert!(err.is_retryable());
}

#[test]
fn error_from_status_500() {
    let err = AnthropicError::from_status(500, HeaderMap::new(), "{}".into());
    assert!(err.is_kind(HttpErrorKind::InternalServer));
    assert!(err.is_retryable());
}

#[test]
fn error_from_status_529() {
    let err = AnthropicError::from_status(529, HeaderMap::new(), "{}".into());
    assert!(err.is_kind(HttpErrorKind::Overloaded));
    assert!(err.is_retryable());
}

#[test]
fn error_from_status_401_not_retryable() {
    let err = AnthropicError::from_status(401, HeaderMap::new(), "{}".into());
    assert!(err.is_kind(HttpErrorKind::Unauthorized));
    assert!(!err.is_retryable());
}

#[test]
fn error_io_is_retryable() {
    let err = AnthropicError::Io(std::io::Error::new(std::io::ErrorKind::ConnectionReset, "reset"));
    assert!(err.is_retryable());
    assert!(err.status().is_none());
}

#[test]
fn error_config_not_retryable() {
    let err = AnthropicError::Config("missing key".into());
    assert!(!err.is_retryable());
}

#[test]
fn error_sse_not_retryable() {
    let err = AnthropicError::Sse("parse error".into());
    assert!(!err.is_retryable());
}

#[test]
fn error_display_includes_message() {
    let err = AnthropicError::from_status(
        403,
        HeaderMap::new(),
        r#"{"error":{"type":"permission_error","message":"not allowed"}}"#.into(),
    );
    let display = err.to_string();
    assert!(display.contains("permission denied"));
    assert!(display.contains("403"));
    assert!(display.contains("not allowed"));
}
