//! Realtime API client for managing WebSocket connections.
//!
//! Provides a high-level interface for connecting to the OpenAI Realtime API,
//! sending client events, receiving server events, and handling function calls.

#![cfg(not(target_arch = "wasm32"))]

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

use base64::Engine;

use super::client_events::ClientEvent;
use super::platform::{self, WsMessage, WsStream};
use super::server_events::ServerEvent;
use super::types::{
    ConversationItem, ItemType, RealtimeModel, RealtimeResponse, RealtimeTool,
    ResponseCreateParams, Session,
};
use crate::error::{AnthropicError, Result};

/// Type alias for an async function call handler.
///
/// Takes a `serde_json::Value` (the parsed arguments) and returns a
/// `serde_json::Value` (the result to send back).
type ToolHandler = Box<
    dyn Fn(serde_json::Value) -> Pin<Box<dyn Future<Output = serde_json::Value> + Send>>
        + Send
        + Sync,
>;

const DEFAULT_BASE_URL: &str = "wss://api.openai.com/v1/realtime";

// ── RealtimeConfig ───────────────────────────────────────────────────

/// Configuration for connecting to the Realtime API.
#[derive(Debug, Clone)]
pub struct RealtimeConfig {
    /// OpenAI API key.
    pub api_key: String,
    /// Model to use for the session.
    pub model: RealtimeModel,
    /// Override the base WebSocket URL (default: `wss://api.openai.com/v1/realtime`).
    pub url: Option<String>,
}

impl RealtimeConfig {
    /// Create a new configuration with the given API key and model.
    pub fn new(api_key: impl Into<String>, model: RealtimeModel) -> Self {
        Self {
            api_key: api_key.into(),
            model,
            url: None,
        }
    }

    /// Override the base WebSocket URL.
    pub fn with_url(mut self, url: impl Into<String>) -> Self {
        self.url = Some(url.into());
        self
    }

    /// Create a configuration by reading `OPENAI_API_KEY` from the environment.
    pub fn from_env(model: RealtimeModel) -> Result<Self> {
        let api_key = std::env::var("OPENAI_API_KEY").map_err(|_| {
            AnthropicError::Config(
                "OPENAI_API_KEY environment variable not set".into(),
            )
        })?;
        Ok(Self::new(api_key, model))
    }
}

// ── RealtimeClient ───────────────────────────────────────────────────

/// High-level client for the OpenAI Realtime API.
///
/// Manages a WebSocket connection and provides methods for sending events,
/// receiving events, and handling function calls from the model.
pub struct RealtimeClient {
    ws: WsStream,
    #[allow(dead_code)]
    config: RealtimeConfig,
    tools: HashMap<String, ToolHandler>,
}

impl RealtimeClient {
    /// Connect to the Realtime API with the given configuration.
    ///
    /// Performs the WebSocket handshake with authentication headers and
    /// returns a connected client ready to send and receive events.
    pub async fn connect(config: RealtimeConfig) -> Result<Self> {
        let base_url = config
            .url
            .as_deref()
            .unwrap_or(DEFAULT_BASE_URL);

        let url = format!("{}?model={}", base_url, config.model);

        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::AUTHORIZATION,
            reqwest::header::HeaderValue::from_str(&format!("Bearer {}", config.api_key))
                .map_err(|e| AnthropicError::Config(format!("invalid API key header: {e}")))?,
        );
        headers.insert(
            "OpenAI-Beta",
            reqwest::header::HeaderValue::from_static("realtime=v1"),
        );

        tracing::debug!(url = %url, model = %config.model, "connecting to realtime API");

        let ws = platform::connect_ws(&url, headers).await?;

        Ok(Self {
            ws,
            config,
            tools: HashMap::new(),
        })
    }

    /// Send a client event to the server.
    ///
    /// Serializes the event to JSON and sends it as a WebSocket text frame.
    pub async fn send(&mut self, event: ClientEvent) -> Result<()> {
        let json = serde_json::to_string(&event)?;
        tracing::debug!(event = %json, "sending client event");
        self.ws.send_text(json).await
    }

    /// Receive the next server event.
    ///
    /// Reads WebSocket frames, deserializes JSON text frames into `ServerEvent`,
    /// skips binary frames, and returns `None` on connection close.
    pub async fn recv(&mut self) -> Option<Result<ServerEvent>> {
        loop {
            let msg = self.ws.recv().await?;
            match msg {
                Ok(WsMessage::Text(text)) => {
                    tracing::debug!(event = %text, "received server event");
                    let event = serde_json::from_str::<ServerEvent>(&text)
                        .map_err(AnthropicError::from);
                    return Some(event);
                }
                Ok(WsMessage::Binary(_)) => {
                    // Skip binary frames; the Realtime API uses JSON text frames.
                    continue;
                }
                Ok(WsMessage::Close(_)) => {
                    return None;
                }
                Err(e) => {
                    return Some(Err(e));
                }
            }
        }
    }

    /// Update the session configuration.
    ///
    /// Sends a `session.update` event with the given session parameters.
    pub async fn update_session(&mut self, session: Session) -> Result<()> {
        self.send(ClientEvent::session_update(session)).await
    }

    /// Send a user text message and trigger a response.
    ///
    /// Creates a conversation item with the text and immediately sends
    /// a `response.create` event.
    pub async fn send_text(&mut self, text: &str) -> Result<()> {
        self.send(ClientEvent::user_message(text)).await?;
        self.send(ClientEvent::create_response()).await
    }

    /// Append raw PCM16 audio bytes to the input buffer.
    ///
    /// The bytes are base64-encoded before sending as an
    /// `input_audio_buffer.append` event.
    pub async fn append_audio(&mut self, pcm16_bytes: &[u8]) -> Result<()> {
        let encoded = base64::engine::general_purpose::STANDARD.encode(pcm16_bytes);
        self.send(ClientEvent::audio_append(encoded)).await
    }

    /// Commit the audio buffer and trigger a response.
    ///
    /// Sends `input_audio_buffer.commit` followed by `response.create`.
    pub async fn commit_audio(&mut self) -> Result<()> {
        self.send(ClientEvent::audio_commit()).await?;
        self.send(ClientEvent::create_response()).await
    }

    /// Create a conversation item.
    pub async fn create_item(&mut self, item: ConversationItem) -> Result<()> {
        self.send(ClientEvent::create_item(item)).await
    }

    /// Trigger a model response, optionally with custom parameters.
    pub async fn create_response(&mut self, params: Option<ResponseCreateParams>) -> Result<()> {
        let event = match params {
            Some(p) => ClientEvent::create_response_with(p),
            None => ClientEvent::create_response(),
        };
        self.send(event).await
    }

    /// Cancel an in-progress response.
    pub async fn cancel_response(&mut self) -> Result<()> {
        self.send(ClientEvent::cancel_response()).await
    }

    /// Register an async function tool handler.
    ///
    /// When the model calls a function with the matching name (from a
    /// `response.done` event), the handler is invoked automatically by
    /// [`handle_function_calls`](Self::handle_function_calls).
    ///
    /// The `definition` should be a `RealtimeTool::function(...)` that
    /// describes the tool's name, description, and parameter schema.
    /// Include it in your `Session.tools` when configuring the session.
    pub fn add_tool<F, Fut>(&mut self, definition: RealtimeTool, handler: F)
    where
        F: Fn(serde_json::Value) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = serde_json::Value> + Send + 'static,
    {
        let name = definition
            .name
            .clone()
            .unwrap_or_default();

        self.tools.insert(
            name,
            Box::new(move |args| Box::pin(handler(args))),
        );
    }

    /// Process function calls from a completed response.
    ///
    /// Iterates over the response output items, finds `FunctionCall` items,
    /// looks up the registered handler by name, executes it with the parsed
    /// arguments, sends a `function_call_output` conversation item, and
    /// triggers a new response.
    pub async fn handle_function_calls(&mut self, response: &RealtimeResponse) -> Result<()> {
        let output = match &response.output {
            Some(items) => items,
            None => return Ok(()),
        };

        for item in output {
            if item.item_type != Some(ItemType::FunctionCall) {
                continue;
            }

            let name = match &item.name {
                Some(n) => n.clone(),
                None => continue,
            };

            let call_id = match &item.call_id {
                Some(id) => id.clone(),
                None => continue,
            };

            let handler = match self.tools.get(&name) {
                Some(h) => h,
                None => {
                    tracing::debug!(
                        function = %name,
                        "no handler registered for function call, skipping"
                    );
                    continue;
                }
            };

            let args_str = item.arguments.as_deref().unwrap_or("{}");
            let args: serde_json::Value = serde_json::from_str(args_str).unwrap_or_default();

            tracing::debug!(function = %name, args = %args, "executing function call");

            let result = handler(args).await;
            let result_str = serde_json::to_string(&result)
                .unwrap_or_else(|_| "null".into());

            tracing::debug!(function = %name, result = %result_str, "function call completed");

            let output_item = ConversationItem::function_call_output(call_id, result_str);
            self.send(ClientEvent::create_item(output_item)).await?;
        }

        // Trigger a new response after sending all function call outputs.
        self.send(ClientEvent::create_response()).await
    }

    /// Close the WebSocket connection gracefully.
    pub async fn close(&mut self) -> Result<()> {
        tracing::debug!("closing realtime WebSocket connection");
        self.ws.close().await
    }
}
