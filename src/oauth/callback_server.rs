//! Localhost HTTP server for receiving OAuth redirect callbacks.
//!
//! Uses raw `tokio::net::TcpListener` to avoid adding a web framework dependency.
//! Binds to `127.0.0.1:0` (auto-port) by default, accepts a single connection,
//! extracts `code` and `state` query parameters, and responds with a success page.

use std::time::Duration;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;
use tracing::{debug, info, warn};

use super::error::{OAuthError, Result};

/// Result from a successful OAuth callback.
#[derive(Debug, Clone)]
pub struct CallbackResult {
    /// The authorization code from the OAuth provider.
    pub code: String,
    /// The state parameter (for CSRF validation).
    pub state: Option<String>,
}

/// Localhost callback server for OAuth redirects.
pub struct CallbackServer {
    listener: TcpListener,
}

impl CallbackServer {
    /// Bind to `127.0.0.1` on the specified port (0 = auto-assign).
    pub async fn bind(port: u16) -> Result<Self> {
        let addr = format!("127.0.0.1:{port}");
        let listener = TcpListener::bind(&addr).await.map_err(|e| {
            OAuthError::CallbackServer(format!("failed to bind to {addr}: {e}"))
        })?;
        info!(addr = %listener.local_addr().unwrap(), "Callback server listening");
        Ok(Self { listener })
    }

    /// Get the local address the server is bound to.
    pub fn local_addr(&self) -> std::net::SocketAddr {
        self.listener.local_addr().unwrap()
    }

    /// Get the port the server is listening on.
    pub fn port(&self) -> u16 {
        self.local_addr().port()
    }

    /// Build the redirect URI for this server.
    pub fn redirect_uri(&self) -> String {
        format!("http://127.0.0.1:{}/callback", self.port())
    }

    /// Wait for a single OAuth callback, then shut down.
    ///
    /// Accepts one TCP connection, parses the HTTP GET request for
    /// `code` and `state` query parameters, responds with a success page,
    /// and returns the result.
    pub async fn wait_for_callback(self, timeout: Duration) -> Result<CallbackResult> {
        tokio::select! {
            result = self.accept_one() => result,
            _ = tokio::time::sleep(timeout) => {
                warn!(timeout_secs = timeout.as_secs(), "OAuth callback timed out");
                Err(OAuthError::CallbackServer(format!(
                    "callback timed out after {} seconds",
                    timeout.as_secs()
                )))
            }
        }
    }

    async fn accept_one(self) -> Result<CallbackResult> {
        let (mut stream, peer) = self.listener.accept().await.map_err(|e| {
            OAuthError::CallbackServer(format!("failed to accept connection: {e}"))
        })?;
        debug!(peer = %peer, "Accepted callback connection");

        // Read the first HTTP request line using buffered reader
        let mut reader = BufReader::new(&mut stream);
        let mut first_line = String::new();
        reader.read_line(&mut first_line).await.map_err(|e| {
            OAuthError::CallbackServer(format!("failed to read request: {e}"))
        })?;
        if first_line.trim().is_empty() {
            return Err(OAuthError::CallbackServer("empty callback request".into()));
        }
        let path = first_line.split_whitespace().nth(1).unwrap_or("/");

        // Parse query parameters using url::Url
        let fake_base = format!("http://localhost{path}");
        let parsed = url::Url::parse(&fake_base).map_err(|e| {
            OAuthError::CallbackServer(format!("failed to parse callback URL: {e}"))
        })?;

        let mut code = None;
        let mut state = None;
        let mut error = None;
        let mut error_description = None;

        for (key, value) in parsed.query_pairs() {
            match key.as_ref() {
                "code" => code = Some(value.to_string()),
                "state" => state = Some(value.to_string()),
                "error" => error = Some(value.to_string()),
                "error_description" => error_description = Some(value.to_string()),
                _ => {}
            }
        }

        // Check for OAuth error
        if let Some(err) = error {
            let desc = error_description.unwrap_or_else(|| "unknown error".to_string());
            let response = error_response(&err, &desc);
            let _ = stream.write_all(response.as_bytes()).await;
            let _ = stream.shutdown().await;
            return Err(OAuthError::OAuth(format!("{err}: {desc}")));
        }

        // Extract code
        let code = code.ok_or_else(|| {
            OAuthError::CallbackServer("missing authorization code in callback".into())
        })?;

        // Send success response
        let response = success_response();
        let _ = stream.write_all(response.as_bytes()).await;
        let _ = stream.shutdown().await;

        info!("OAuth callback received successfully");

        Ok(CallbackResult { code, state })
    }
}

fn success_response() -> String {
    let body = r#"<!DOCTYPE html>
<html><head><title>Authentication Successful</title>
<style>body{font-family:system-ui;display:flex;justify-content:center;align-items:center;
min-height:100vh;margin:0;background:#1a1a2e;color:#e0e0e0}
.c{text-align:center;padding:2rem}h1{color:#34d399}</style>
<script>setTimeout(()=>window.close(),3000)</script>
</head><body><div class="c"><h1>Authentication Successful</h1>
<p>You can close this window and return to the terminal.</p></div></body></html>"#;

    format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.len(),
        body
    )
}

fn error_response(error: &str, description: &str) -> String {
    let safe_error = escape_html(error);
    let safe_description = escape_html(description);
    let body = format!(
        r#"<!DOCTYPE html>
<html><head><title>Authentication Failed</title>
<style>body{{font-family:system-ui;display:flex;justify-content:center;align-items:center;
min-height:100vh;margin:0;background:#1a1a2e;color:#e0e0e0}}
.c{{text-align:center;padding:2rem}}h1{{color:#f87171}}</style>
</head><body><div class="c"><h1>Authentication Failed</h1>
<p>{safe_error}: {safe_description}</p></div></body></html>"#
    );

    format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.len(),
        body
    )
}

fn escape_html(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_bind_auto_port() {
        let server = CallbackServer::bind(0).await.unwrap();
        assert!(server.port() > 0);
    }

    #[tokio::test]
    async fn test_redirect_uri_format() {
        let server = CallbackServer::bind(0).await.unwrap();
        let uri = server.redirect_uri();
        assert!(uri.starts_with("http://127.0.0.1:"));
        assert!(uri.ends_with("/callback"));
    }

    #[tokio::test]
    async fn test_callback_receives_code() {
        let server = CallbackServer::bind(0).await.unwrap();
        let port = server.port();

        // Spawn a client that sends a fake callback
        tokio::spawn(async move {
            // Small delay to let the server start accepting
            tokio::time::sleep(Duration::from_millis(50)).await;
            let client = reqwest::Client::new();
            let _ = client
                .get(format!(
                    "http://127.0.0.1:{port}/callback?code=test_code&state=test_state"
                ))
                .send()
                .await;
        });

        let result = server
            .wait_for_callback(Duration::from_secs(5))
            .await
            .unwrap();
        assert_eq!(result.code, "test_code");
        assert_eq!(result.state.as_deref(), Some("test_state"));
    }

    #[tokio::test]
    async fn test_callback_timeout() {
        let server = CallbackServer::bind(0).await.unwrap();
        let result = server
            .wait_for_callback(Duration::from_millis(100))
            .await;
        assert!(result.is_err());
    }
}
