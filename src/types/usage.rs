use serde::{Deserialize, Serialize};

use super::service_tier::UsageServiceTier;

/// Token usage statistics for a message.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Usage {
    pub input_tokens: u64,
    pub output_tokens: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cache_creation_input_tokens: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cache_read_input_tokens: Option<u64>,
    /// Per-TTL breakdown of cache creation tokens.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cache_creation: Option<CacheCreation>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub server_tool_use: Option<ServerToolUsage>,
    /// The service tier used for this request.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub service_tier: Option<UsageServiceTier>,
    /// Geographic region where inference was performed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inference_geo: Option<String>,
}

/// Per-TTL breakdown of cache creation tokens.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CacheCreation {
    /// Tokens used to create the 5-minute cache entry.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ephemeral_5m_input_tokens: Option<u64>,
    /// Tokens used to create the 1-hour cache entry.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ephemeral_1h_input_tokens: Option<u64>,
}

/// Server tool usage breakdown.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ServerToolUsage {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub web_search_requests: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub web_fetch_requests: Option<u64>,
}

/// Usage delta reported in streaming message_delta events.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MessageDeltaUsage {
    pub output_tokens: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_tokens: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cache_creation_input_tokens: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cache_read_input_tokens: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub server_tool_use: Option<ServerToolUsage>,
}
