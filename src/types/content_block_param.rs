use serde::{Deserialize, Serialize};

use super::cache_control::CacheControl;
use super::document::DocumentBlockParam;
use super::image::ImageBlockParam;
use super::server_tool_use::ServerToolUseBlockParam;
use super::text::TextBlockParam;
use super::thinking::{RedactedThinkingBlock, ThinkingBlockParam};
use super::tool_result::ToolResultBlockParam;
use super::tool_use::ToolUseBlockParam;

/// A content block param in a request message.
///
/// Includes both user-provided blocks (text, image, document, tool_result)
/// and pass-back blocks for multi-turn conversations with server tools.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ContentBlockParam {
    #[serde(rename = "text")]
    Text(TextBlockParam),

    #[serde(rename = "image")]
    Image(ImageBlockParam),

    #[serde(rename = "document")]
    Document(DocumentBlockParam),

    #[serde(rename = "thinking")]
    Thinking(ThinkingBlockParam),

    #[serde(rename = "redacted_thinking")]
    RedactedThinking(RedactedThinkingBlock),

    #[serde(rename = "tool_use")]
    ToolUse(ToolUseBlockParam),

    #[serde(rename = "tool_result")]
    ToolResult(ToolResultBlockParam),

    #[serde(rename = "server_tool_use")]
    ServerToolUse(ServerToolUseBlockParam),

    // --- Multi-turn pass-back variants for server tool results ---

    #[serde(rename = "web_search_tool_result")]
    WebSearchToolResult(WebSearchToolResultBlockParam),

    #[serde(rename = "web_fetch_tool_result")]
    WebFetchToolResult(WebFetchToolResultBlockParam),

    #[serde(rename = "code_execution_tool_result")]
    CodeExecutionToolResult(CodeExecutionToolResultBlockParam),

    #[serde(rename = "bash_code_execution_tool_result")]
    BashCodeExecutionToolResult(BashCodeExecutionToolResultBlockParam),

    #[serde(rename = "text_editor_code_execution_tool_result")]
    TextEditorCodeExecutionToolResult(TextEditorCodeExecutionToolResultBlockParam),

    #[serde(rename = "tool_search_tool_result")]
    ToolSearchToolResult(ToolSearchToolResultBlockParam),

    #[serde(rename = "container_upload")]
    ContainerUpload(ContainerUploadBlockParam),

    #[serde(rename = "search_result")]
    SearchResult(SearchResultBlockParam),
}

impl ContentBlockParam {
    /// Create a text block.
    pub fn text(text: impl Into<String>) -> Self {
        Self::Text(TextBlockParam {
            text: text.into(),
            cache_control: None,
            citations: None,
        })
    }

    /// Create an image block from base64 data.
    pub fn image_base64(media_type: super::image::MediaType, data: impl Into<String>) -> Self {
        Self::Image(ImageBlockParam {
            source: super::image::ImageSource::Base64 {
                media_type,
                data: data.into(),
            },
            cache_control: None,
        })
    }

    /// Create an image block from a URL.
    pub fn image_url(url: impl Into<String>) -> Self {
        Self::Image(ImageBlockParam {
            source: super::image::ImageSource::Url { url: url.into() },
            cache_control: None,
        })
    }

    /// Create a tool result block.
    pub fn tool_result(
        tool_use_id: impl Into<String>,
        content: impl Into<String>,
    ) -> Self {
        Self::ToolResult(ToolResultBlockParam {
            tool_use_id: tool_use_id.into(),
            content: super::tool_result::ToolResultContentParam::Text(content.into()),
            is_error: None,
            cache_control: None,
        })
    }
}

// --- Param types for server tool result pass-back ---

/// Web search tool result param for multi-turn conversations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSearchToolResultBlockParam {
    pub tool_use_id: String,
    pub content: serde_json::Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_error: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
}

/// Web fetch tool result param for multi-turn conversations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebFetchToolResultBlockParam {
    pub tool_use_id: String,
    pub content: serde_json::Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_error: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
}

/// Code execution tool result param for multi-turn conversations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeExecutionToolResultBlockParam {
    pub tool_use_id: String,
    pub content: serde_json::Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub return_value: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_error: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
}

/// Bash code execution tool result param for multi-turn conversations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BashCodeExecutionToolResultBlockParam {
    pub tool_use_id: String,
    pub content: serde_json::Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_error: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
}

/// Text editor code execution tool result param for multi-turn conversations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextEditorCodeExecutionToolResultBlockParam {
    pub tool_use_id: String,
    pub content: serde_json::Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_error: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
}

/// Tool search tool result param for multi-turn conversations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSearchToolResultBlockParam {
    pub tool_use_id: String,
    pub content: serde_json::Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_error: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
}

/// Container upload block param for multi-turn conversations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerUploadBlockParam {
    pub file_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
}

/// Search result block param for multi-turn conversations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResultBlockParam {
    pub source: serde_json::Value,
    pub content: serde_json::Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
}
