//! OpenAI Realtime API types and events.
//!
//! This module provides typed representations of all client and server events
//! for the OpenAI Realtime API WebSocket protocol. Events are serialized as
//! JSON text frames over the WebSocket connection.
//!
//! # Event flow
//!
//! 1. Connect via WebSocket to `wss://api.openai.com/v1/realtime?model=<model>`
//! 2. Receive [`ServerEvent::SessionCreated`] with the default session config
//! 3. Optionally send [`ClientEvent::SessionUpdate`] to configure the session
//! 4. Send conversation items and trigger responses
//! 5. Receive streaming deltas and lifecycle events
//!
//! # Example
//!
//! ```no_run
//! use anthropic_rs::realtime::{ClientEvent, ServerEvent, Session, Voice};
//!
//! # fn example() {
//! // Build a session update
//! let event = ClientEvent::session_update(Session {
//!     voice: Some(Voice::Marin),
//!     instructions: Some("Be helpful.".into()),
//!     ..Default::default()
//! });
//!
//! // Serialize to JSON for sending over WebSocket
//! let json = serde_json::to_string(&event).unwrap();
//!
//! // Deserialize a server event from received JSON
//! let server_event: ServerEvent = serde_json::from_str(&json).unwrap_or_else(|_| {
//!     panic!("unknown event");
//! });
//! # }
//! ```

pub mod audio;
pub mod client_events;
pub mod conversation;
pub mod error;
pub mod server_events;
pub mod session;
pub mod types;

#[cfg(not(target_arch = "wasm32"))]
pub mod client;
#[cfg(not(target_arch = "wasm32"))]
pub(crate) mod platform;

// Re-export primary types at module level for convenience.
pub use client_events::ClientEvent;
pub use conversation::ConversationState;
pub use error::RealtimeErrorKind;
pub use server_events::ServerEvent;
pub use session::SessionState;
pub use types::{
    AudioFormat, ContentPart, ContentType, Conversation, ConversationItem, Eagerness,
    InputAudioNoiseReduction, InputAudioTranscription, InputTokenDetails, ItemStatus, ItemType,
    MaxOutputTokens, Modality, NoiseReductionType, OutputTokenDetails, RateLimit, RealtimeError,
    RealtimeModel, RealtimeResponse, RealtimeTool, RealtimeUsage, ResponseCreateParams,
    ResponseError, ResponseStatus, ResponseStatusDetails, Role, Session, StatusReason, Tracing,
    TracingConfig, TranscriptionError, TurnDetection, Voice,
};

#[cfg(not(target_arch = "wasm32"))]
pub use client::{RealtimeClient, RealtimeConfig};
