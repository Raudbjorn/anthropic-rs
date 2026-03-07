//! Server-sent events from the OpenAI Realtime API.
//!
//! These events are received over the WebSocket connection as JSON text frames.

use serde::{Deserialize, Serialize};

use super::types::{
    ContentPart, Conversation, ConversationItem, RateLimit, RealtimeError, RealtimeResponse,
    Session, TranscriptionError,
};

/// All server-sent events, discriminated by the `type` field.
///
/// Each variant maps to an OpenAI Realtime API server event.
/// Unrecognized event types deserialize as [`Unknown`](Self::Unknown)
/// for forward compatibility (new event types may be added by the API).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ServerEvent {
    // ── Error ────────────────────────────────────────────────────────

    /// An error occurred.
    #[serde(rename = "error")]
    Error {
        event_id: String,
        error: RealtimeError,
    },

    // ── Session ──────────────────────────────────────────────────────

    /// Session created (first event after connection).
    #[serde(rename = "session.created")]
    SessionCreated {
        event_id: String,
        session: Session,
    },

    /// Session configuration updated.
    #[serde(rename = "session.updated")]
    SessionUpdated {
        event_id: String,
        session: Session,
    },

    // ── Conversation ─────────────────────────────────────────────────

    /// Conversation created.
    #[serde(rename = "conversation.created")]
    ConversationCreated {
        event_id: String,
        conversation: Conversation,
    },

    // ── Conversation Items ───────────────────────────────────────────

    /// Item created in the conversation.
    #[serde(rename = "conversation.item.created")]
    ConversationItemCreated {
        event_id: String,
        item: ConversationItem,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        previous_item_id: Option<String>,
    },

    /// Item deleted from the conversation.
    #[serde(rename = "conversation.item.deleted")]
    ConversationItemDeleted {
        event_id: String,
        item_id: String,
    },

    /// Item truncated (audio cut after interruption).
    #[serde(rename = "conversation.item.truncated")]
    ConversationItemTruncated {
        event_id: String,
        item_id: String,
        content_index: u32,
        audio_end_ms: u32,
    },

    /// Item retrieved from server state.
    #[serde(rename = "conversation.item.retrieved")]
    ConversationItemRetrieved {
        event_id: String,
        item: ConversationItem,
    },

    // ── Input Audio Transcription ────────────────────────────────────

    /// Input audio transcription completed.
    #[serde(rename = "conversation.item.input_audio_transcription.completed")]
    InputAudioTranscriptionCompleted {
        event_id: String,
        item_id: String,
        content_index: u32,
        transcript: String,
    },

    /// Input audio transcription failed.
    #[serde(rename = "conversation.item.input_audio_transcription.failed")]
    InputAudioTranscriptionFailed {
        event_id: String,
        item_id: String,
        content_index: u32,
        error: TranscriptionError,
    },

    // ── Input Audio Buffer ───────────────────────────────────────────

    /// Input audio buffer committed as a user turn.
    #[serde(rename = "input_audio_buffer.committed")]
    InputAudioBufferCommitted {
        event_id: String,
        item_id: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        previous_item_id: Option<String>,
    },

    /// Input audio buffer cleared.
    #[serde(rename = "input_audio_buffer.cleared")]
    InputAudioBufferCleared { event_id: String },

    /// Speech detected in the input audio buffer (VAD).
    #[serde(rename = "input_audio_buffer.speech_started")]
    InputAudioBufferSpeechStarted {
        event_id: String,
        item_id: String,
        audio_start_ms: u32,
    },

    /// Speech stopped in the input audio buffer (VAD).
    #[serde(rename = "input_audio_buffer.speech_stopped")]
    InputAudioBufferSpeechStopped {
        event_id: String,
        item_id: String,
        audio_end_ms: u32,
    },

    // ── Response Lifecycle ───────────────────────────────────────────

    /// Response generation started.
    #[serde(rename = "response.created")]
    ResponseCreated {
        event_id: String,
        response: RealtimeResponse,
    },

    /// Response generation completed (or failed/cancelled).
    #[serde(rename = "response.done")]
    ResponseDone {
        event_id: String,
        response: RealtimeResponse,
    },

    // ── Response Output Items ────────────────────────────────────────

    /// New output item added to the response.
    #[serde(rename = "response.output_item.added")]
    ResponseOutputItemAdded {
        event_id: String,
        response_id: String,
        output_index: u32,
        item: ConversationItem,
    },

    /// Output item completed.
    #[serde(rename = "response.output_item.done")]
    ResponseOutputItemDone {
        event_id: String,
        response_id: String,
        output_index: u32,
        item: ConversationItem,
    },

    // ── Response Content Parts ───────────────────────────────────────

    /// Content part added to an output item.
    #[serde(rename = "response.content_part.added")]
    ResponseContentPartAdded {
        event_id: String,
        response_id: String,
        item_id: String,
        output_index: u32,
        content_index: u32,
        part: ContentPart,
    },

    /// Content part completed.
    #[serde(rename = "response.content_part.done")]
    ResponseContentPartDone {
        event_id: String,
        response_id: String,
        item_id: String,
        output_index: u32,
        content_index: u32,
        part: ContentPart,
    },

    // ── Response Text ────────────────────────────────────────────────

    /// Incremental text delta.
    #[serde(rename = "response.text.delta")]
    ResponseTextDelta {
        event_id: String,
        response_id: String,
        item_id: String,
        output_index: u32,
        content_index: u32,
        delta: String,
    },

    /// Text generation completed for this content part.
    #[serde(rename = "response.text.done")]
    ResponseTextDone {
        event_id: String,
        response_id: String,
        item_id: String,
        output_index: u32,
        content_index: u32,
        text: String,
    },

    // ── Response Audio ───────────────────────────────────────────────

    /// Incremental audio delta (base64-encoded).
    #[serde(rename = "response.audio.delta")]
    ResponseAudioDelta {
        event_id: String,
        response_id: String,
        item_id: String,
        output_index: u32,
        content_index: u32,
        /// Base64-encoded audio bytes.
        delta: String,
    },

    /// Audio generation completed for this content part.
    #[serde(rename = "response.audio.done")]
    ResponseAudioDone {
        event_id: String,
        response_id: String,
        item_id: String,
        output_index: u32,
        content_index: u32,
    },

    // ── Response Audio Transcript ────────────────────────────────────

    /// Incremental audio transcript delta.
    #[serde(rename = "response.audio_transcript.delta")]
    ResponseAudioTranscriptDelta {
        event_id: String,
        response_id: String,
        item_id: String,
        output_index: u32,
        content_index: u32,
        delta: String,
    },

    /// Audio transcript completed.
    #[serde(rename = "response.audio_transcript.done")]
    ResponseAudioTranscriptDone {
        event_id: String,
        response_id: String,
        item_id: String,
        output_index: u32,
        content_index: u32,
        transcript: String,
    },

    // ── Response Function Call Arguments ──────────────────────────────

    /// Incremental function call arguments delta.
    #[serde(rename = "response.function_call_arguments.delta")]
    ResponseFunctionCallArgumentsDelta {
        event_id: String,
        response_id: String,
        item_id: String,
        output_index: u32,
        call_id: String,
        delta: String,
    },

    /// Function call arguments completed.
    #[serde(rename = "response.function_call_arguments.done")]
    ResponseFunctionCallArgumentsDone {
        event_id: String,
        response_id: String,
        item_id: String,
        output_index: u32,
        call_id: String,
        arguments: String,
    },

    // ── Rate Limits ──────────────────────────────────────────────────

    /// Rate limit information updated.
    #[serde(rename = "rate_limits.updated")]
    RateLimitsUpdated {
        event_id: String,
        rate_limits: Vec<RateLimit>,
    },

    // ── Output Audio Buffer (WebRTC/SIP, but may appear on WS) ──────

    /// Output audio buffer playback started.
    #[serde(rename = "output_audio_buffer.started")]
    OutputAudioBufferStarted {
        event_id: String,
        response_id: String,
    },

    /// Output audio buffer playback stopped.
    #[serde(rename = "output_audio_buffer.stopped")]
    OutputAudioBufferStopped {
        event_id: String,
        response_id: String,
    },

    /// Output audio buffer cleared.
    #[serde(rename = "output_audio_buffer.cleared")]
    OutputAudioBufferCleared {
        event_id: String,
        response_id: String,
    },

    /// An unrecognized event type (forward compatibility).
    ///
    /// Payload data is not preserved; use this variant to detect and skip
    /// unknown events without failing deserialization.
    #[serde(other)]
    Unknown,
}

impl ServerEvent {
    /// Get the `event_id` from any server event.
    pub fn event_id(&self) -> &str {
        match self {
            Self::Error { event_id, .. }
            | Self::SessionCreated { event_id, .. }
            | Self::SessionUpdated { event_id, .. }
            | Self::ConversationCreated { event_id, .. }
            | Self::ConversationItemCreated { event_id, .. }
            | Self::ConversationItemDeleted { event_id, .. }
            | Self::ConversationItemTruncated { event_id, .. }
            | Self::ConversationItemRetrieved { event_id, .. }
            | Self::InputAudioTranscriptionCompleted { event_id, .. }
            | Self::InputAudioTranscriptionFailed { event_id, .. }
            | Self::InputAudioBufferCommitted { event_id, .. }
            | Self::InputAudioBufferCleared { event_id, .. }
            | Self::InputAudioBufferSpeechStarted { event_id, .. }
            | Self::InputAudioBufferSpeechStopped { event_id, .. }
            | Self::ResponseCreated { event_id, .. }
            | Self::ResponseDone { event_id, .. }
            | Self::ResponseOutputItemAdded { event_id, .. }
            | Self::ResponseOutputItemDone { event_id, .. }
            | Self::ResponseContentPartAdded { event_id, .. }
            | Self::ResponseContentPartDone { event_id, .. }
            | Self::ResponseTextDelta { event_id, .. }
            | Self::ResponseTextDone { event_id, .. }
            | Self::ResponseAudioDelta { event_id, .. }
            | Self::ResponseAudioDone { event_id, .. }
            | Self::ResponseAudioTranscriptDelta { event_id, .. }
            | Self::ResponseAudioTranscriptDone { event_id, .. }
            | Self::ResponseFunctionCallArgumentsDelta { event_id, .. }
            | Self::ResponseFunctionCallArgumentsDone { event_id, .. }
            | Self::RateLimitsUpdated { event_id, .. }
            | Self::OutputAudioBufferStarted { event_id, .. }
            | Self::OutputAudioBufferStopped { event_id, .. }
            | Self::OutputAudioBufferCleared { event_id, .. } => event_id,
            Self::Unknown => "",
        }
    }

    /// Check if this is an error event.
    pub fn is_error(&self) -> bool {
        matches!(self, Self::Error { .. })
    }

    /// Check if this is a response completion event.
    pub fn is_response_done(&self) -> bool {
        matches!(self, Self::ResponseDone { .. })
    }

    /// Extract the response from `response.done` or `response.created`.
    pub fn into_response(self) -> Option<RealtimeResponse> {
        match self {
            Self::ResponseDone { response, .. } | Self::ResponseCreated { response, .. } => {
                Some(response)
            }
            _ => None,
        }
    }

    /// Extract the text delta, if this is a `response.text.delta` event.
    pub fn text_delta(&self) -> Option<&str> {
        match self {
            Self::ResponseTextDelta { delta, .. } => Some(delta),
            _ => None,
        }
    }

    /// Extract the audio delta (base64), if this is a `response.audio.delta` event.
    pub fn audio_delta(&self) -> Option<&str> {
        match self {
            Self::ResponseAudioDelta { delta, .. } => Some(delta),
            _ => None,
        }
    }

    /// Extract the audio transcript delta, if this is a `response.audio_transcript.delta` event.
    pub fn audio_transcript_delta(&self) -> Option<&str> {
        match self {
            Self::ResponseAudioTranscriptDelta { delta, .. } => Some(delta),
            _ => None,
        }
    }

    /// Extract function call arguments delta, if this is that event.
    pub fn function_call_arguments_delta(&self) -> Option<&str> {
        match self {
            Self::ResponseFunctionCallArgumentsDelta { delta, .. } => Some(delta),
            _ => None,
        }
    }
}
