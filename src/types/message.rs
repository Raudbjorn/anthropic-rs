use serde::{Deserialize, Serialize};

use super::container::Container;
use super::content_block::ContentBlock;
use super::content_block_param::ContentBlockParam;
use super::model::Model;
use super::stop_reason::StopReason;
use super::usage::Usage;

/// A response message from the API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    #[serde(rename = "type")]
    pub message_type: String,
    pub role: String,
    pub content: Vec<ContentBlock>,
    pub model: Model,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stop_reason: Option<StopReason>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stop_sequence: Option<String>,
    pub usage: Usage,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub container: Option<Container>,
}

impl Message {
    /// Extract all text content blocks concatenated.
    pub fn text(&self) -> String {
        self.content
            .iter()
            .filter_map(|b| b.as_text())
            .collect::<Vec<_>>()
            .join("")
    }

    /// Extract all thinking blocks concatenated.
    pub fn thinking(&self) -> String {
        self.content
            .iter()
            .filter_map(|b| b.as_thinking())
            .collect::<Vec<_>>()
            .join("")
    }

    /// Get all tool use blocks.
    pub fn tool_uses(&self) -> Vec<&super::tool_use::ToolUseBlock> {
        self.content
            .iter()
            .filter_map(|b| b.as_tool_use())
            .collect()
    }
}

/// Role for a message parameter.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    User,
    Assistant,
}

/// Message content — either a simple string or structured blocks.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MessageContent {
    Text(String),
    Blocks(Vec<ContentBlockParam>),
}

impl From<&str> for MessageContent {
    fn from(s: &str) -> Self {
        Self::Text(s.to_owned())
    }
}

impl From<String> for MessageContent {
    fn from(s: String) -> Self {
        Self::Text(s)
    }
}

impl From<Vec<ContentBlockParam>> for MessageContent {
    fn from(blocks: Vec<ContentBlockParam>) -> Self {
        Self::Blocks(blocks)
    }
}

/// A message parameter for input.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageParam {
    pub role: Role,
    pub content: MessageContent,
}

impl MessageParam {
    /// Create a user message with text content.
    pub fn user(content: impl Into<MessageContent>) -> Self {
        Self {
            role: Role::User,
            content: content.into(),
        }
    }

    /// Create an assistant message with text content.
    pub fn assistant(content: impl Into<MessageContent>) -> Self {
        Self {
            role: Role::Assistant,
            content: content.into(),
        }
    }
}
