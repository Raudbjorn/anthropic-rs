use serde::{Deserialize, Serialize};

use super::cache_control::CacheControl;

/// Text editor server tool configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextEditorTool {
    #[serde(rename = "type")]
    pub tool_type: String,
    pub name: String,
    /// Maximum number of characters the tool can handle.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_characters: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
}

impl TextEditorTool {
    pub fn new() -> Self {
        Self {
            tool_type: "text_editor_20250728".to_owned(),
            name: "str_replace_based_edit_tool".to_owned(),
            max_characters: None,
            cache_control: None,
        }
    }
}

impl Default for TextEditorTool {
    fn default() -> Self {
        Self::new()
    }
}
