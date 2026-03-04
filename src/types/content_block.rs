use serde::{Deserialize, Serialize};

use super::code_execution::{BashCodeExecutionToolResultBlock, CodeExecutionToolResultBlock, TextEditorCodeExecutionToolResultBlock};
use super::container::ContainerUploadBlock;
use super::server_tool_use::ServerToolUseBlock;
use super::text::TextBlock;
use super::thinking::{RedactedThinkingBlock, ThinkingBlock};
use super::tool_search::ToolSearchToolResultBlock;
use super::tool_use::ToolUseBlock;
use super::web_fetch::WebFetchToolResultBlock;
use super::web_search::WebSearchToolResultBlock;

/// A content block in a response message.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ContentBlock {
    #[serde(rename = "text")]
    Text(TextBlock),

    #[serde(rename = "thinking")]
    Thinking(ThinkingBlock),

    #[serde(rename = "redacted_thinking")]
    RedactedThinking(RedactedThinkingBlock),

    #[serde(rename = "tool_use")]
    ToolUse(ToolUseBlock),

    #[serde(rename = "server_tool_use")]
    ServerToolUse(ServerToolUseBlock),

    #[serde(rename = "web_search_tool_result")]
    WebSearchToolResult(WebSearchToolResultBlock),

    #[serde(rename = "web_fetch_tool_result")]
    WebFetchToolResult(WebFetchToolResultBlock),

    #[serde(rename = "code_execution_tool_result")]
    CodeExecutionToolResult(CodeExecutionToolResultBlock),

    #[serde(rename = "bash_code_execution_tool_result")]
    BashCodeExecutionToolResult(BashCodeExecutionToolResultBlock),

    #[serde(rename = "text_editor_code_execution_tool_result")]
    TextEditorCodeExecutionToolResult(TextEditorCodeExecutionToolResultBlock),

    #[serde(rename = "tool_search_tool_result")]
    ToolSearchToolResult(ToolSearchToolResultBlock),

    #[serde(rename = "container_upload")]
    ContainerUpload(ContainerUploadBlock),
}

impl ContentBlock {
    /// Extract text if this is a text block.
    pub fn as_text(&self) -> Option<&str> {
        match self {
            Self::Text(b) => Some(&b.text),
            _ => None,
        }
    }

    /// Extract thinking text if this is a thinking block.
    pub fn as_thinking(&self) -> Option<&str> {
        match self {
            Self::Thinking(b) => Some(&b.thinking),
            _ => None,
        }
    }

    /// Extract tool use if this is a tool use block.
    pub fn as_tool_use(&self) -> Option<&ToolUseBlock> {
        match self {
            Self::ToolUse(b) => Some(b),
            _ => None,
        }
    }
}
