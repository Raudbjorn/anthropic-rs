use serde::{Deserialize, Serialize};

use super::cache_control::CacheControl;

/// Tool search server tool configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSearchTool {
    #[serde(rename = "type")]
    pub tool_type: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
}

impl ToolSearchTool {
    /// Create a BM25 tool search tool.
    pub fn bm25() -> Self {
        Self {
            tool_type: "tool_search_tool_bm25".to_owned(),
            name: "tool_search".to_owned(),
            cache_control: None,
        }
    }

    /// Create a regex tool search tool.
    pub fn regex() -> Self {
        Self {
            tool_type: "tool_search_tool_regex".to_owned(),
            name: "tool_search".to_owned(),
            cache_control: None,
        }
    }

    /// Create a BM25 tool search tool (default).
    pub fn new() -> Self {
        Self::bm25()
    }
}

impl Default for ToolSearchTool {
    fn default() -> Self {
        Self::new()
    }
}

/// Tool search result block in a response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSearchToolResultBlock {
    pub tool_use_id: String,
    pub content: serde_json::Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_error: Option<bool>,
}
