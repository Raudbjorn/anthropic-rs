use serde::{Deserialize, Serialize};

use super::cache_control::CacheControl;

/// Bash server tool configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BashTool {
    #[serde(rename = "type")]
    pub tool_type: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
}

impl BashTool {
    pub fn new() -> Self {
        Self {
            tool_type: "bash_20250124".to_owned(),
            name: "bash".to_owned(),
            cache_control: None,
        }
    }
}

impl Default for BashTool {
    fn default() -> Self {
        Self::new()
    }
}
