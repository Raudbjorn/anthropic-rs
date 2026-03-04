use serde::{Deserialize, Serialize};

use super::cache_control::CacheControl;

/// Web fetch server tool configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebFetchTool {
    #[serde(rename = "type")]
    pub tool_type: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_uses: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allowed_domains: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blocked_domains: Option<Vec<String>>,
    /// Whether to include citations in the response.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub citations: Option<bool>,
    /// Maximum content tokens to fetch per page.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_content_tokens: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
}

impl WebFetchTool {
    pub fn new() -> Self {
        Self {
            tool_type: "web_fetch_20260209".to_owned(),
            name: "web_fetch".to_owned(),
            max_uses: None,
            allowed_domains: None,
            blocked_domains: None,
            citations: None,
            max_content_tokens: None,
            cache_control: None,
        }
    }
}

impl Default for WebFetchTool {
    fn default() -> Self {
        Self::new()
    }
}

/// Web fetch tool result block in a response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebFetchToolResultBlock {
    pub tool_use_id: String,
    pub content: serde_json::Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_error: Option<bool>,
}
