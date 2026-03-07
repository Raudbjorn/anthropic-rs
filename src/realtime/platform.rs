//! WebSocket platform abstraction for the Realtime API.
//!
//! Provides a native WebSocket connection using `tokio-tungstenite`.
//! WASM support is not yet implemented.

#![cfg(not(target_arch = "wasm32"))]

use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, tungstenite, MaybeTlsStream};

use crate::error::{AnthropicError, Result};
use super::error::RealtimeErrorKind;

// ── Message types ────────────────────────────────────────────────────

/// A close frame with code and reason.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct CloseFrame {
    pub code: u16,
    pub reason: String,
}

/// WebSocket message types relevant to the Realtime API.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum WsMessage {
    /// UTF-8 text frame.
    Text(String),
    /// Binary data frame.
    Binary(Vec<u8>),
    /// Connection close frame.
    Close(Option<CloseFrame>),
}

// ── WsStream ─────────────────────────────────────────────────────────

/// A wrapper around the tokio-tungstenite WebSocket stream.
pub(crate) struct WsStream {
    inner: tokio_tungstenite::WebSocketStream<MaybeTlsStream<TcpStream>>,
}

impl WsStream {
    /// Send a text message over the WebSocket.
    pub async fn send_text(&mut self, msg: String) -> Result<()> {
        self.inner
            .send(tungstenite::Message::Text(msg))
            .await
            .map_err(|e| {
                AnthropicError::Io(std::io::Error::new(std::io::ErrorKind::BrokenPipe, e))
            })
    }

    /// Receive the next message from the WebSocket.
    ///
    /// Returns `None` when the connection is closed cleanly.
    /// Ping/Pong frames are handled automatically by tungstenite;
    /// binary frames are passed through.
    pub async fn recv(&mut self) -> Option<Result<WsMessage>> {
        loop {
            let msg = self.inner.next().await?;
            match msg {
                Ok(tungstenite::Message::Text(text)) => {
                    return Some(Ok(WsMessage::Text(text)));
                }
                Ok(tungstenite::Message::Binary(data)) => {
                    return Some(Ok(WsMessage::Binary(data)));
                }
                Ok(tungstenite::Message::Close(frame)) => {
                    let close = frame.map(|f| CloseFrame {
                        code: f.code.into(),
                        reason: f.reason.into_owned(),
                    });
                    return Some(Ok(WsMessage::Close(close)));
                }
                Ok(tungstenite::Message::Ping(_) | tungstenite::Message::Pong(_)) => {
                    // Handled internally by tungstenite; skip.
                    continue;
                }
                Ok(tungstenite::Message::Frame(_)) => {
                    // Raw frames are not expected; skip.
                    continue;
                }
                Err(tungstenite::Error::ConnectionClosed | tungstenite::Error::AlreadyClosed) => {
                    return None;
                }
                Err(e) => {
                    return Some(Err(AnthropicError::Io(std::io::Error::new(
                        std::io::ErrorKind::ConnectionReset,
                        e,
                    ))));
                }
            }
        }
    }

    /// Close the WebSocket connection gracefully.
    pub async fn close(&mut self) -> Result<()> {
        self.inner.close(None).await.map_err(|e| {
            AnthropicError::Io(std::io::Error::new(std::io::ErrorKind::ConnectionAborted, e))
        })
    }
}

// ── connect_ws ───────────────────────────────────────────────────────

/// Connect to a WebSocket endpoint with the given URL and headers.
///
/// Builds an HTTP upgrade request with the supplied headers (e.g. Authorization)
/// and establishes a TLS WebSocket connection via `tokio-tungstenite`.
pub(crate) async fn connect_ws(
    url: &str,
    headers: reqwest::header::HeaderMap,
) -> Result<WsStream> {
    use tungstenite::http::Request;

    let mut builder = Request::builder().uri(url);

    for (name, value) in &headers {
        builder = builder.header(name.as_str(), value.as_bytes());
    }

    let request = builder
        .body(())
        .map_err(|e| AnthropicError::Config(format!("failed to build WebSocket request: {e}")))?;

    tracing::debug!(url = %url, "connecting to realtime WebSocket");

    let (ws_stream, _response) = connect_async(request).await.map_err(|e| {
        AnthropicError::Realtime(RealtimeErrorKind::ConnectionFailed(
            format!("WebSocket connection failed: {e}"),
        ))
    })?;

    tracing::debug!("realtime WebSocket connected");

    Ok(WsStream { inner: ws_stream })
}
