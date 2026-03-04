use serde::{Deserialize, Serialize};

/// How the model should use tools.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ToolChoice {
    /// Model decides whether to use tools.
    #[serde(rename = "auto")]
    Auto {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        disable_parallel_tool_use: Option<bool>,
    },
    /// Model must use at least one tool.
    #[serde(rename = "any")]
    Any {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        disable_parallel_tool_use: Option<bool>,
    },
    /// Model must use the specified tool.
    #[serde(rename = "tool")]
    Tool {
        name: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        disable_parallel_tool_use: Option<bool>,
    },
    /// Model must not use tools.
    #[serde(rename = "none")]
    None,
}

impl ToolChoice {
    pub fn auto() -> Self {
        Self::Auto { disable_parallel_tool_use: None }
    }

    pub fn any() -> Self {
        Self::Any { disable_parallel_tool_use: None }
    }

    pub fn tool(name: impl Into<String>) -> Self {
        Self::Tool { name: name.into(), disable_parallel_tool_use: None }
    }

    pub fn none() -> Self {
        Self::None
    }
}
