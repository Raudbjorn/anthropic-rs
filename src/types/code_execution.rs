use serde::{Deserialize, Serialize};

use super::cache_control::CacheControl;

/// Code execution server tool configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeExecutionTool {
    #[serde(rename = "type")]
    pub tool_type: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_uses: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
}

impl CodeExecutionTool {
    pub fn new() -> Self {
        Self {
            tool_type: "code_execution_20260120".to_owned(),
            name: "code_execution".to_owned(),
            max_uses: None,
            cache_control: None,
        }
    }
}

impl Default for CodeExecutionTool {
    fn default() -> Self {
        Self::new()
    }
}

/// Code execution tool result block in a response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeExecutionToolResultBlock {
    pub tool_use_id: String,
    pub content: CodeExecutionContent,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub return_value: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_error: Option<bool>,
}

/// Content of a code execution result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeExecutionContent {
    #[serde(rename = "type")]
    pub content_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stdout: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stderr: Option<String>,
}

/// Bash code execution tool result block.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BashCodeExecutionToolResultBlock {
    pub tool_use_id: String,
    pub content: serde_json::Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_error: Option<bool>,
}

/// Text editor code execution tool result block.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextEditorCodeExecutionToolResultBlock {
    pub tool_use_id: String,
    pub content: serde_json::Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_error: Option<bool>,
}
