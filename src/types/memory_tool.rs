use serde::{Deserialize, Serialize};

use super::cache_control::CacheControl;

/// Memory server tool configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryTool {
    #[serde(rename = "type")]
    pub tool_type: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
}

impl MemoryTool {
    pub fn new() -> Self {
        Self {
            tool_type: "memory_20250818".to_owned(),
            name: "memory".to_owned(),
            cache_control: None,
        }
    }
}

impl Default for MemoryTool {
    fn default() -> Self {
        Self::new()
    }
}
