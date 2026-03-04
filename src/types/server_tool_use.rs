use serde::{Deserialize, Serialize};

/// A server-executed tool use block in a response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerToolUseBlock {
    pub id: String,
    pub name: String,
    pub input: serde_json::Value,
    /// Who invoked this tool (direct model call or another server tool).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub caller: Option<ServerToolCaller>,
}

/// Identifies who invoked a server tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ServerToolCaller {
    /// Tool invocation directly from the model.
    #[serde(rename = "direct_caller")]
    DirectCaller,
    /// Tool invoked by another server tool (e.g., code execution calling bash).
    #[serde(rename = "server_tool_caller")]
    ServerToolCaller {
        /// ID of the parent tool use.
        tool_use_id: String,
    },
}

/// A server tool use block param for multi-turn.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerToolUseBlockParam {
    pub id: String,
    pub name: String,
    pub input: serde_json::Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<super::cache_control::CacheControl>,
}
