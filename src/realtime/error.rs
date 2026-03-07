//! Realtime API-specific error types.

use std::fmt;

/// Errors specific to the Realtime WebSocket API.
#[derive(Debug)]
pub enum RealtimeErrorKind {
    /// WebSocket connection failed.
    ConnectionFailed(String),
    /// WebSocket connection was closed.
    ConnectionClosed {
        code: u16,
        reason: String,
    },
    /// The server sent an error event.
    ServerError {
        error_type: String,
        message: String,
        code: Option<String>,
        param: Option<String>,
        event_id: Option<String>,
    },
    /// Failed to parse an event received from the server.
    InvalidEvent(String),
    /// Not connected to the Realtime API.
    NotConnected,
}

impl fmt::Display for RealtimeErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ConnectionFailed(msg) => write!(f, "connection failed: {msg}"),
            Self::ConnectionClosed { code, reason } => {
                write!(f, "connection closed ({code}): {reason}")
            }
            Self::ServerError { error_type, message, .. } => {
                write!(f, "server error ({error_type}): {message}")
            }
            Self::InvalidEvent(msg) => write!(f, "invalid event: {msg}"),
            Self::NotConnected => write!(f, "not connected"),
        }
    }
}

impl RealtimeErrorKind {
    /// Create from a server error event.
    pub fn from_server_error(error: &super::types::RealtimeError) -> Self {
        Self::ServerError {
            error_type: error.error_type.clone(),
            message: error.message.clone(),
            code: error.code.clone(),
            param: error.param.clone(),
            event_id: error.event_id.clone(),
        }
    }
}
