use serde::{Deserialize, Serialize};

use super::cache_control::CacheControl;

/// Computer use server tool configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputerUseTool {
    #[serde(rename = "type")]
    pub tool_type: String,
    pub name: String,
    /// Display width in pixels.
    pub display_width_px: u32,
    /// Display height in pixels.
    pub display_height_px: u32,
    /// Display number (for multi-display setups).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_number: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
}

impl ComputerUseTool {
    /// Create a computer use tool with the latest version.
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            tool_type: "computer_20251124".to_owned(),
            name: "computer".to_owned(),
            display_width_px: width,
            display_height_px: height,
            display_number: None,
            cache_control: None,
        }
    }

    /// Create with a specific version string.
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.tool_type = version.into();
        self
    }

    pub fn with_display_number(mut self, num: u32) -> Self {
        self.display_number = Some(num);
        self
    }
}
