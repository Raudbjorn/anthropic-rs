//! Client-sent events for the OpenAI Realtime API.
//!
//! These events are serialized to JSON and sent over the WebSocket connection
//! from the client to the server.

use serde::{Deserialize, Serialize};

use super::types::{ConversationItem, ResponseCreateParams, Session};

/// All client-sent events, discriminated by the `type` field.
///
/// Each variant serializes with a `"type"` tag matching the OpenAI API
/// event names (e.g. `"session.update"`, `"response.create"`).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ClientEvent {
    // ── Session ──────────────────────────────────────────────────────

    /// Update the session configuration.
    #[serde(rename = "session.update")]
    SessionUpdate {
        session: Session,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        event_id: Option<String>,
    },

    // ── Input Audio Buffer ───────────────────────────────────────────

    /// Append base64-encoded audio to the input buffer.
    #[serde(rename = "input_audio_buffer.append")]
    InputAudioBufferAppend {
        /// Base64-encoded audio bytes.
        audio: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        event_id: Option<String>,
    },

    /// Commit the current input audio buffer as a user turn.
    #[serde(rename = "input_audio_buffer.commit")]
    InputAudioBufferCommit {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        event_id: Option<String>,
    },

    /// Clear the input audio buffer.
    #[serde(rename = "input_audio_buffer.clear")]
    InputAudioBufferClear {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        event_id: Option<String>,
    },

    // ── Output Audio Buffer ──────────────────────────────────────────

    /// Clear the output audio buffer (WebRTC/SIP only, but supported on WS).
    #[serde(rename = "output_audio_buffer.clear")]
    OutputAudioBufferClear {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        event_id: Option<String>,
    },

    // ── Conversation Items ───────────────────────────────────────────

    /// Add an item to the conversation.
    #[serde(rename = "conversation.item.create")]
    ConversationItemCreate {
        item: ConversationItem,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        event_id: Option<String>,
        /// Insert after this item ID. Use `"root"` for the beginning.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        previous_item_id: Option<String>,
    },

    /// Delete an item from the conversation.
    #[serde(rename = "conversation.item.delete")]
    ConversationItemDelete {
        item_id: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        event_id: Option<String>,
    },

    /// Retrieve an item from the server's conversation state.
    #[serde(rename = "conversation.item.retrieve")]
    ConversationItemRetrieve {
        item_id: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        event_id: Option<String>,
    },

    /// Truncate a model's audio response after an interruption.
    #[serde(rename = "conversation.item.truncate")]
    ConversationItemTruncate {
        item_id: String,
        content_index: u32,
        audio_end_ms: u32,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        event_id: Option<String>,
    },

    // ── Response ─────────────────────────────────────────────────────

    /// Trigger the model to generate a response.
    #[serde(rename = "response.create")]
    ResponseCreate {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        response: Option<ResponseCreateParams>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        event_id: Option<String>,
    },

    /// Cancel an in-progress response.
    #[serde(rename = "response.cancel")]
    ResponseCancel {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        response_id: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        event_id: Option<String>,
    },
}

// ── Convenience constructors ─────────────────────────────────────────

impl ClientEvent {
    /// Create a `session.update` event.
    pub fn session_update(session: Session) -> Self {
        Self::SessionUpdate {
            session,
            event_id: None,
        }
    }

    /// Create an `input_audio_buffer.append` event.
    pub fn audio_append(audio_base64: impl Into<String>) -> Self {
        Self::InputAudioBufferAppend {
            audio: audio_base64.into(),
            event_id: None,
        }
    }

    /// Create an `input_audio_buffer.commit` event.
    pub fn audio_commit() -> Self {
        Self::InputAudioBufferCommit { event_id: None }
    }

    /// Create an `input_audio_buffer.clear` event.
    pub fn audio_clear() -> Self {
        Self::InputAudioBufferClear { event_id: None }
    }

    /// Create a `conversation.item.create` event with a user text message.
    pub fn user_message(text: impl Into<String>) -> Self {
        Self::ConversationItemCreate {
            item: ConversationItem::user_message(text),
            event_id: None,
            previous_item_id: None,
        }
    }

    /// Create a `conversation.item.create` event with a custom item.
    pub fn create_item(item: ConversationItem) -> Self {
        Self::ConversationItemCreate {
            item,
            event_id: None,
            previous_item_id: None,
        }
    }

    /// Create a `conversation.item.delete` event.
    pub fn delete_item(item_id: impl Into<String>) -> Self {
        Self::ConversationItemDelete {
            item_id: item_id.into(),
            event_id: None,
        }
    }

    /// Create a `conversation.item.truncate` event.
    pub fn truncate_item(item_id: impl Into<String>, content_index: u32, audio_end_ms: u32) -> Self {
        Self::ConversationItemTruncate {
            item_id: item_id.into(),
            content_index,
            audio_end_ms,
            event_id: None,
        }
    }

    /// Create a `response.create` event with default params.
    pub fn create_response() -> Self {
        Self::ResponseCreate {
            response: None,
            event_id: None,
        }
    }

    /// Create a `response.create` event with custom params.
    pub fn create_response_with(params: ResponseCreateParams) -> Self {
        Self::ResponseCreate {
            response: Some(params),
            event_id: None,
        }
    }

    /// Create a `response.cancel` event.
    pub fn cancel_response() -> Self {
        Self::ResponseCancel {
            response_id: None,
            event_id: None,
        }
    }

    /// Set the `event_id` on this event (builder-style).
    pub fn with_event_id(mut self, id: impl Into<String>) -> Self {
        let id = Some(id.into());
        match &mut self {
            Self::SessionUpdate { event_id, .. }
            | Self::InputAudioBufferAppend { event_id, .. }
            | Self::InputAudioBufferCommit { event_id, .. }
            | Self::InputAudioBufferClear { event_id, .. }
            | Self::OutputAudioBufferClear { event_id, .. }
            | Self::ConversationItemCreate { event_id, .. }
            | Self::ConversationItemDelete { event_id, .. }
            | Self::ConversationItemRetrieve { event_id, .. }
            | Self::ConversationItemTruncate { event_id, .. }
            | Self::ResponseCreate { event_id, .. }
            | Self::ResponseCancel { event_id, .. } => *event_id = id,
        }
        self
    }
}
