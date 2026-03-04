use serde::{Deserialize, Serialize};

/// A tool use block in a response (model invoking a tool).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolUseBlock {
    pub id: String,
    pub name: String,
    pub input: serde_json::Value,
}

/// A tool use block param for assistant messages in multi-turn.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolUseBlockParam {
    pub id: String,
    pub name: String,
    pub input: serde_json::Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<super::cache_control::CacheControl>,
}

/// Delta for streaming partial JSON input.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputJsonDelta {
    pub partial_json: String,
}
