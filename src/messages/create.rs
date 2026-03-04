use serde::Serialize;

use crate::types::message::{MessageParam, MessageContent};
use crate::types::metadata::Metadata;
use crate::types::model::Model;
use crate::types::output_config::OutputConfig;
use crate::types::service_tier::ServiceTier;
use crate::types::thinking::ThinkingConfig;
use crate::types::tool::ToolUnion;
use crate::types::tool_choice::ToolChoice;
/// Parameters for creating a message.
#[derive(Debug, Clone, Serialize)]
pub struct MessageCreateParams {
    pub model: Model,
    pub messages: Vec<MessageParam>,
    pub max_tokens: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<SystemPrompt>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_k: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_sequences: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<ToolUnion>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<ToolChoice>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking: Option<ThinkingConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Metadata>,
    #[serde(rename = "output_config", skip_serializing_if = "Option::is_none")]
    pub output: Option<OutputConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_tier: Option<ServiceTier>,
    /// Container ID for code execution.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub container: Option<String>,
    /// Geographic region for inference processing.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inference_geo: Option<String>,
    /// Streaming flag — set internally by stream methods.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) stream: Option<bool>,
}

/// System prompt — either a simple string or structured blocks.
#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum SystemPrompt {
    Text(String),
    Blocks(Vec<crate::types::content_block_param::ContentBlockParam>),
}

impl From<&str> for SystemPrompt {
    fn from(s: &str) -> Self {
        Self::Text(s.to_owned())
    }
}

impl From<String> for SystemPrompt {
    fn from(s: String) -> Self {
        Self::Text(s)
    }
}

impl MessageCreateParams {
    /// Create a builder for MessageCreateParams.
    pub fn builder(model: Model, max_tokens: u64) -> MessageCreateParamsBuilder {
        MessageCreateParamsBuilder {
            model,
            max_tokens,
            messages: Vec::new(),
            system: None,
            temperature: None,
            top_p: None,
            top_k: None,
            stop_sequences: None,
            tools: None,
            tool_choice: None,
            thinking: None,
            metadata: None,
            output: None,
            service_tier: None,
            container: None,
            inference_geo: None,
        }
    }
}

/// Builder for `MessageCreateParams`.
#[derive(Debug)]
pub struct MessageCreateParamsBuilder {
    model: Model,
    max_tokens: u64,
    messages: Vec<MessageParam>,
    system: Option<SystemPrompt>,
    temperature: Option<f64>,
    top_p: Option<f64>,
    top_k: Option<u64>,
    stop_sequences: Option<Vec<String>>,
    tools: Option<Vec<ToolUnion>>,
    tool_choice: Option<ToolChoice>,
    thinking: Option<ThinkingConfig>,
    metadata: Option<Metadata>,
    output: Option<OutputConfig>,
    service_tier: Option<ServiceTier>,
    container: Option<String>,
    inference_geo: Option<String>,
}

impl MessageCreateParamsBuilder {
    pub fn messages(mut self, messages: Vec<MessageParam>) -> Self {
        self.messages = messages;
        self
    }

    pub fn message(mut self, message: MessageParam) -> Self {
        self.messages.push(message);
        self
    }

    /// Convenience: add a user message with text.
    pub fn user(self, content: impl Into<MessageContent>) -> Self {
        self.message(MessageParam::user(content))
    }

    /// Convenience: add an assistant message with text.
    pub fn assistant(self, content: impl Into<MessageContent>) -> Self {
        self.message(MessageParam::assistant(content))
    }

    pub fn system(mut self, system: impl Into<SystemPrompt>) -> Self {
        self.system = Some(system.into());
        self
    }

    pub fn temperature(mut self, temperature: f64) -> Self {
        self.temperature = Some(temperature);
        self
    }

    pub fn top_p(mut self, top_p: f64) -> Self {
        self.top_p = Some(top_p);
        self
    }

    pub fn top_k(mut self, top_k: u64) -> Self {
        self.top_k = Some(top_k);
        self
    }

    pub fn stop_sequences(mut self, sequences: Vec<String>) -> Self {
        self.stop_sequences = Some(sequences);
        self
    }

    pub fn tools(mut self, tools: Vec<ToolUnion>) -> Self {
        self.tools = Some(tools);
        self
    }

    pub fn tool(mut self, tool: impl Into<ToolUnion>) -> Self {
        self.tools.get_or_insert_with(Vec::new).push(tool.into());
        self
    }

    pub fn tool_choice(mut self, choice: ToolChoice) -> Self {
        self.tool_choice = Some(choice);
        self
    }

    pub fn thinking(mut self, config: ThinkingConfig) -> Self {
        self.thinking = Some(config);
        self
    }

    pub fn metadata(mut self, metadata: Metadata) -> Self {
        self.metadata = Some(metadata);
        self
    }

    pub fn output(mut self, output: OutputConfig) -> Self {
        self.output = Some(output);
        self
    }

    pub fn service_tier(mut self, tier: ServiceTier) -> Self {
        self.service_tier = Some(tier);
        self
    }

    /// Set the container ID for code execution.
    pub fn container(mut self, container: impl Into<String>) -> Self {
        self.container = Some(container.into());
        self
    }

    /// Set geographic region for inference processing.
    pub fn inference_geo(mut self, geo: impl Into<String>) -> Self {
        self.inference_geo = Some(geo.into());
        self
    }

    pub fn build(self) -> MessageCreateParams {
        MessageCreateParams {
            model: self.model,
            messages: self.messages,
            max_tokens: self.max_tokens,
            system: self.system,
            temperature: self.temperature,
            top_p: self.top_p,
            top_k: self.top_k,
            stop_sequences: self.stop_sequences,
            tools: self.tools,
            tool_choice: self.tool_choice,
            thinking: self.thinking,
            metadata: self.metadata,
            output: self.output,
            service_tier: self.service_tier,
            container: self.container,
            inference_geo: self.inference_geo,
            stream: None,
        }
    }
}
