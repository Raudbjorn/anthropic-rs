//! Synchronous (blocking) wrapper around the async `Anthropic` client.
//!
//! Requires the `blocking` feature flag. Creates a dedicated tokio runtime.

use crate::batches::{BatchCreateParams, BatchListParams, DeletedMessageBatch, MessageBatch};
use crate::config::{RetryConfig, Timeout};
use crate::error::{AnthropicError, Result};
use crate::messages::{MessageCountTokensParams, MessageCreateParams, MessageTokensCount};
use crate::models_api::{ModelInfo, ModelListParams};
use crate::page::Page;
use crate::types::message::Message;

/// A blocking (synchronous) Anthropic client.
///
/// Wraps the async client with a dedicated tokio runtime.
pub struct BlockingAnthropic {
    inner: crate::client::Anthropic,
    runtime: tokio::runtime::Runtime,
}

impl BlockingAnthropic {
    /// Create from an existing async client.
    pub fn new(inner: crate::client::Anthropic) -> Result<Self> {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .map_err(|e| AnthropicError::Config(format!("failed to create runtime: {e}")))?;
        Ok(Self { inner, runtime })
    }

    /// Create from environment variables.
    pub fn from_env() -> Result<Self> {
        let inner = crate::client::Anthropic::from_env()?;
        Self::new(inner)
    }

    /// Builder.
    pub fn builder() -> BlockingAnthropicBuilder {
        BlockingAnthropicBuilder {
            inner: crate::client::Anthropic::builder(),
        }
    }

    pub fn messages_create(&self, params: MessageCreateParams) -> Result<Message> {
        self.runtime.block_on(self.inner.messages_create(params))
    }

    pub fn messages_count_tokens(
        &self,
        params: MessageCountTokensParams,
    ) -> Result<MessageTokensCount> {
        self.runtime
            .block_on(self.inner.messages_count_tokens(params))
    }

    pub fn batches_create(&self, params: BatchCreateParams) -> Result<MessageBatch> {
        self.runtime.block_on(self.inner.batches_create(params))
    }

    pub fn batches_retrieve(&self, batch_id: &str) -> Result<MessageBatch> {
        self.runtime.block_on(self.inner.batches_retrieve(batch_id))
    }

    pub fn batches_list(&self, params: BatchListParams) -> Result<Page<MessageBatch>> {
        self.runtime.block_on(self.inner.batches_list(params))
    }

    pub fn batches_cancel(&self, batch_id: &str) -> Result<MessageBatch> {
        self.runtime.block_on(self.inner.batches_cancel(batch_id))
    }

    pub fn batches_delete(&self, batch_id: &str) -> Result<DeletedMessageBatch> {
        self.runtime.block_on(self.inner.batches_delete(batch_id))
    }

    pub fn models_retrieve(&self, model_id: &str) -> Result<ModelInfo> {
        self.runtime.block_on(self.inner.models_retrieve(model_id))
    }

    pub fn models_list(&self, params: ModelListParams) -> Result<Page<ModelInfo>> {
        self.runtime.block_on(self.inner.models_list(params))
    }
}

/// Builder for `BlockingAnthropic`.
pub struct BlockingAnthropicBuilder {
    inner: crate::client::AnthropicBuilder,
}

impl BlockingAnthropicBuilder {
    pub fn api_key(mut self, key: impl Into<String>) -> Self {
        self.inner = self.inner.api_key(key);
        self
    }

    pub fn base_url(mut self, url: impl Into<String>) -> Self {
        self.inner = self.inner.base_url(url);
        self
    }

    pub fn timeout(mut self, timeout: Timeout) -> Self {
        self.inner = self.inner.timeout(timeout);
        self
    }

    pub fn retry_config(mut self, config: RetryConfig) -> Self {
        self.inner = self.inner.retry_config(config);
        self
    }

    pub fn build(self) -> Result<BlockingAnthropic> {
        let inner = self.inner.build()?;
        BlockingAnthropic::new(inner)
    }
}
