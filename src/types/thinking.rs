use serde::{Deserialize, Serialize};

/// Extended thinking block in a response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThinkingBlock {
    pub thinking: String,
    pub signature: String,
}

/// Thinking block param for including in assistant turns.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThinkingBlockParam {
    pub thinking: String,
    pub signature: String,
}

/// Redacted thinking block (content not visible).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedactedThinkingBlock {
    pub data: String,
}

/// Configuration for extended thinking.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ThinkingConfig {
    /// Enable thinking with a specific token budget.
    #[serde(rename = "enabled")]
    Enabled { budget_tokens: u64 },
    /// Disable thinking entirely.
    #[serde(rename = "disabled")]
    Disabled,
    /// Let the model adaptively decide thinking budget.
    #[serde(rename = "adaptive")]
    Adaptive,
}

/// Delta for streaming thinking updates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThinkingDelta {
    pub thinking: String,
}

/// Delta for streaming signature updates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignatureDelta {
    pub signature: String,
}
