use serde::{Deserialize, Serialize};

use super::cache_control::CacheControl;
use super::metadata::UserLocation;

/// Web search server tool configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSearchTool {
    #[serde(rename = "type")]
    pub tool_type: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_uses: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allowed_domains: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blocked_domains: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_location: Option<UserLocation>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
}

impl WebSearchTool {
    /// Create a web search tool with the latest version.
    pub fn new() -> Self {
        Self {
            tool_type: "web_search_20260209".to_owned(),
            name: "web_search".to_owned(),
            max_uses: None,
            allowed_domains: None,
            blocked_domains: None,
            user_location: None,
            cache_control: None,
        }
    }
}

impl Default for WebSearchTool {
    fn default() -> Self {
        Self::new()
    }
}

/// Web search tool result block in a response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSearchToolResultBlock {
    pub tool_use_id: String,
    pub content: Vec<WebSearchResult>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_error: Option<bool>,
}

/// An individual web search result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSearchResult {
    #[serde(rename = "type")]
    pub result_type: String,
    pub url: String,
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub encrypted_content: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub page_age: Option<String>,
}
