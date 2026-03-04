use serde::{Deserialize, Serialize};

use crate::types::message::MessageParam;
use crate::types::model::Model;
use crate::types::tool::ToolUnion;
use crate::types::thinking::ThinkingConfig;
use super::create::SystemPrompt;

/// Parameters for counting tokens in a message.
#[derive(Debug, Clone, Serialize)]
pub struct MessageCountTokensParams {
    pub model: Model,
    pub messages: Vec<MessageParam>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<SystemPrompt>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<ToolUnion>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking: Option<ThinkingConfig>,
}

/// Response from token counting.
#[derive(Debug, Clone, Deserialize)]
pub struct MessageTokensCount {
    pub input_tokens: u64,
}

impl MessageCountTokensParams {
    pub fn new(model: Model, messages: Vec<MessageParam>) -> Self {
        Self {
            model,
            messages,
            system: None,
            tools: None,
            thinking: None,
        }
    }
}
