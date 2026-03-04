use serde::{Deserialize, Serialize};

use super::cache_control::CacheControl;

/// Tool result content — can be text or image.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ToolResultContent {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image")]
    Image {
        source: super::image::ImageSource,
    },
}

/// Tool result block param (user returning tool output).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResultBlockParam {
    pub tool_use_id: String,
    pub content: ToolResultContentParam,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_error: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
}

/// The content of a tool result — either a simple string or structured blocks.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ToolResultContentParam {
    Text(String),
    Blocks(Vec<ToolResultContent>),
}
