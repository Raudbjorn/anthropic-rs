use serde::{Deserialize, Serialize};

use crate::types::content_block::ContentBlock;
use crate::types::message::Message;
use crate::types::stop_reason::StopReason;
use crate::types::usage::MessageDeltaUsage;

/// A raw streaming event from the Messages API.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum RawMessageStreamEvent {
    #[serde(rename = "message_start")]
    MessageStart { message: Message },

    #[serde(rename = "content_block_start")]
    ContentBlockStart {
        index: usize,
        content_block: ContentBlock,
    },

    #[serde(rename = "content_block_delta")]
    ContentBlockDelta {
        index: usize,
        delta: ContentBlockDelta,
    },

    #[serde(rename = "content_block_stop")]
    ContentBlockStop { index: usize },

    #[serde(rename = "message_delta")]
    MessageDelta {
        delta: MessageDeltaBody,
        usage: MessageDeltaUsage,
    },

    #[serde(rename = "message_stop")]
    MessageStop,

    #[serde(rename = "ping")]
    Ping,

    #[serde(rename = "error")]
    Error { error: StreamError },
}

/// Delta content within a content block.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ContentBlockDelta {
    #[serde(rename = "text_delta")]
    TextDelta { text: String },

    #[serde(rename = "input_json_delta")]
    InputJsonDelta { partial_json: String },

    #[serde(rename = "thinking_delta")]
    ThinkingDelta { thinking: String },

    #[serde(rename = "signature_delta")]
    SignatureDelta { signature: String },

    #[serde(rename = "citations_delta")]
    CitationsDelta {
        citation: crate::types::citation::TextCitation,
    },
}

/// The delta body in a message_delta event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageDeltaBody {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stop_reason: Option<StopReason>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stop_sequence: Option<String>,
    /// Container info (returned when code execution is used).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub container: Option<crate::types::container::Container>,
}

/// An error within a stream.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamError {
    #[serde(rename = "type")]
    pub error_type: String,
    pub message: String,
}
