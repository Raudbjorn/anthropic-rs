use std::pin::Pin;
use std::sync::Arc;

use bytes::Bytes;
use futures_core::Stream;
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::Client;
use tracing::{debug, warn};

use crate::backends::{AnthropicBackend, Backend, BackendRequest};
use crate::batches::{
    BatchCreateParams, BatchListParams, DeletedMessageBatch, MessageBatch,
    MessageBatchIndividualResponse,
};
use crate::config::{RetryConfig, Timeout};
use crate::error::{AnthropicError, Result};
use crate::http::retry::{calculate_backoff, is_retryable_status, parse_retry_after};
use crate::messages::{MessageCountTokensParams, MessageCreateParams, MessageTokensCount};
use crate::models_api::{ModelInfo, ModelListParams};
use crate::page::Page;
use crate::streaming::MessageStream;
use crate::types::message::Message;

/// The Anthropic API client.
///
/// Supports multiple cloud backends via the [`Backend`] trait:
/// - Direct Anthropic API (default)
/// - AWS Bedrock (`feature = "bedrock"`)
/// - Google Vertex AI (`feature = "vertex"`)
/// - Azure AI Foundry (`feature = "foundry"`)
#[derive(Clone)]
pub struct Anthropic {
    backend: Arc<dyn Backend>,
    http: Client,
    retry_config: RetryConfig,
}

impl std::fmt::Debug for Anthropic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Anthropic")
            .field("retry_config", &self.retry_config)
            .finish_non_exhaustive()
    }
}

/// Builder for constructing an `Anthropic` client.
pub struct AnthropicBuilder {
    api_key: Option<String>,
    auth_token: Option<String>,
    base_url: Option<String>,
    betas: Option<Vec<String>>,
    custom_backend: Option<Arc<dyn Backend>>,
    timeout: Timeout,
    retry_config: RetryConfig,
}

impl AnthropicBuilder {
    pub fn new() -> Self {
        Self {
            api_key: None,
            auth_token: None,
            base_url: None,
            betas: None,
            custom_backend: None,
            timeout: Timeout::default(),
            retry_config: RetryConfig::default(),
        }
    }

    /// Set the API key for the direct Anthropic API.
    ///
    /// Ignored when a custom backend is set via [`backend`](Self::backend).
    /// Falls back to `ANTHROPIC_API_KEY` env var if not set.
    pub fn api_key(mut self, key: impl Into<String>) -> Self {
        self.api_key = Some(key.into());
        self
    }

    /// Set the bearer auth token for the direct Anthropic API.
    ///
    /// Used instead of (or alongside) an API key. Ignored when a custom
    /// backend is set via [`backend`](Self::backend).
    /// Falls back to `ANTHROPIC_AUTH_TOKEN` env var if not set.
    pub fn auth_token(mut self, token: impl Into<String>) -> Self {
        self.auth_token = Some(token.into());
        self
    }

    /// Set the base URL for the direct Anthropic API.
    ///
    /// Ignored when a custom backend is set via [`backend`](Self::backend).
    /// Falls back to `ANTHROPIC_BASE_URL` env var, then `https://api.anthropic.com`.
    pub fn base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = Some(url.into());
        self
    }

    /// Override the beta features sent in the `anthropic-beta` header.
    ///
    /// By default, `interleaved-thinking-2025-05-14` and
    /// `code-execution-2025-05-22` are enabled.
    /// Ignored when a custom backend is set via [`backend`](Self::backend).
    pub fn betas(mut self, betas: Vec<String>) -> Self {
        self.betas = Some(betas);
        self
    }

    /// Add a single beta feature to the default set.
    ///
    /// Ignored when a custom backend is set via [`backend`](Self::backend).
    pub fn beta(mut self, beta: impl Into<String>) -> Self {
        self.betas
            .get_or_insert_with(Vec::new)
            .push(beta.into());
        self
    }

    /// Use a custom backend (e.g., Bedrock, Vertex, Foundry).
    ///
    /// When set, `api_key`, `auth_token`, `base_url`, and `betas` are ignored —
    /// the backend handles its own authentication and URL construction.
    pub fn backend(mut self, backend: impl Backend + 'static) -> Self {
        self.custom_backend = Some(Arc::new(backend));
        self
    }

    /// Set HTTP timeouts.
    ///
    /// Defaults: 30s connect, 600s (10 min) request.
    pub fn timeout(mut self, timeout: Timeout) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set retry configuration.
    ///
    /// Defaults: 2 retries, 500ms initial backoff, 8s max backoff.
    pub fn retry_config(mut self, config: RetryConfig) -> Self {
        self.retry_config = config;
        self
    }

    pub fn build(self) -> Result<Anthropic> {
        let backend: Arc<dyn Backend> = if let Some(b) = self.custom_backend {
            b
        } else {
            // Build an AnthropicBackend from the builder fields + env vars
            let mut bb = AnthropicBackend::builder();

            if let Some(key) = self.api_key {
                bb = bb.api_key(key);
            } else if let Ok(key) = std::env::var("ANTHROPIC_API_KEY") {
                bb = bb.api_key(key);
            }

            if let Some(token) = self.auth_token {
                bb = bb.auth_token(token);
            } else if let Ok(token) = std::env::var("ANTHROPIC_AUTH_TOKEN") {
                bb = bb.auth_token(token);
            }

            if let Some(url) = self.base_url {
                bb = bb.base_url(url);
            } else if let Ok(url) = std::env::var("ANTHROPIC_BASE_URL") {
                bb = bb.base_url(url);
            }

            if let Some(betas) = self.betas {
                bb = bb.betas(betas);
            }

            Arc::new(bb.build()?)
        };

        let http = Client::builder()
            .connect_timeout(self.timeout.connect)
            .timeout(self.timeout.request)
            .build()
            .map_err(AnthropicError::Http)?;

        Ok(Anthropic {
            backend,
            http,
            retry_config: self.retry_config,
        })
    }
}

impl Default for AnthropicBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl Anthropic {
    /// Create a builder for configuring the client.
    pub fn builder() -> AnthropicBuilder {
        AnthropicBuilder::new()
    }

    /// Create a client from environment variables (direct Anthropic API).
    pub fn from_env() -> Result<Self> {
        Self::builder().build()
    }

    // ── Messages ──────────────────────────────────────────────────────

    /// Create a message (non-streaming).
    pub async fn messages_create(&self, params: MessageCreateParams) -> Result<Message> {
        let body = serde_json::to_value(&params)?;
        self.post_json(&["v1", "messages"], body).await
    }

    /// Create a message with streaming.
    pub async fn messages_create_stream(
        &self,
        mut params: MessageCreateParams,
    ) -> Result<MessageStream> {
        params.stream = Some(true);
        let body = serde_json::to_value(&params)?;

        let mut req = BackendRequest {
            method: reqwest::Method::POST,
            path_segments: vec!["v1".into(), "messages".into()],
            query_params: vec![],
            headers: HeaderMap::new(),
            body: Some(body),
        };

        // Let the backend transform and authorize the request
        req = self.backend.prepare_request(req)?;
        let url = self.resolve_backend_url(&req)?;
        req = self.backend.authorize_request(req, &url)?;

        let mut http_req = self.http.request(req.method, url.as_str()).headers(req.headers);
        if let Some(ref body) = req.body {
            http_req = http_req.json(body);
        }

        let response = http_req.send().await.map_err(AnthropicError::Http)?;

        let status = response.status().as_u16();
        if status != 200 {
            let headers = response.headers().clone();
            let body_text = response.text().await.unwrap_or_default();
            return Err(AnthropicError::from_status(status, headers, body_text));
        }

        let byte_stream = response.bytes_stream();
        let transformer = self.backend.stream_transformer();
        Ok(MessageStream::new(Box::pin(byte_stream), transformer))
    }

    /// Count tokens for a message without generating a response.
    pub async fn messages_count_tokens(
        &self,
        params: MessageCountTokensParams,
    ) -> Result<MessageTokensCount> {
        let body = serde_json::to_value(&params)?;
        self.post_json(&["v1", "messages", "count_tokens"], body)
            .await
    }

    // ── Batches ───────────────────────────────────────────────────────

    /// Create a message batch.
    pub async fn batches_create(&self, params: BatchCreateParams) -> Result<MessageBatch> {
        let body = serde_json::to_value(&params)?;
        self.post_json(&["v1", "messages", "batches"], body).await
    }

    /// Retrieve a batch by ID.
    pub async fn batches_retrieve(&self, batch_id: &str) -> Result<MessageBatch> {
        self.get_json(&["v1", "messages", "batches", batch_id], &[])
            .await
    }

    /// List batches.
    pub async fn batches_list(&self, params: BatchListParams) -> Result<Page<MessageBatch>> {
        let mut query: Vec<(&str, String)> = Vec::new();
        if let Some(limit) = params.limit {
            query.push(("limit", limit.to_string()));
        }
        if let Some(ref before) = params.before_id {
            query.push(("before_id", before.clone()));
        }
        if let Some(ref after) = params.after_id {
            query.push(("after_id", after.clone()));
        }
        self.get_json(&["v1", "messages", "batches"], &query).await
    }

    /// Cancel a batch.
    pub async fn batches_cancel(&self, batch_id: &str) -> Result<MessageBatch> {
        self.post_json(
            &["v1", "messages", "batches", batch_id, "cancel"],
            serde_json::Value::Object(serde_json::Map::new()),
        )
        .await
    }

    /// Delete a batch.
    pub async fn batches_delete(&self, batch_id: &str) -> Result<DeletedMessageBatch> {
        self.delete_json(&["v1", "messages", "batches", batch_id])
            .await
    }

    /// Stream batch results as JSONL.
    pub async fn batches_results_stream(
        &self,
        batch_id: &str,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<MessageBatchIndividualResponse>> + Send>>> {
        let path_segments: Vec<&str> =
            vec!["v1", "messages", "batches", batch_id, "results"];

        let mut req = BackendRequest {
            method: reqwest::Method::GET,
            path_segments: path_segments.iter().map(|s| s.to_string()).collect(),
            query_params: vec![],
            headers: HeaderMap::new(),
            body: None,
        };

        req = self.backend.prepare_request(req)?;
        let url = self.resolve_backend_url(&req)?;
        req = self.backend.authorize_request(req, &url)?;

        let response = self
            .http
            .get(url.as_str())
            .headers(req.headers)
            .send()
            .await
            .map_err(AnthropicError::Http)?;

        let status = response.status().as_u16();
        if status != 200 {
            let hdrs = response.headers().clone();
            let body = response.text().await.unwrap_or_default();
            return Err(AnthropicError::from_status(status, hdrs, body));
        }

        let byte_stream = response.bytes_stream();
        let line_stream = jsonl_stream(byte_stream);
        Ok(Box::pin(line_stream))
    }

    // ── Models ────────────────────────────────────────────────────────

    /// Retrieve model info.
    pub async fn models_retrieve(&self, model_id: &str) -> Result<ModelInfo> {
        self.get_json(&["v1", "models", model_id], &[]).await
    }

    /// List available models.
    pub async fn models_list(&self, params: ModelListParams) -> Result<Page<ModelInfo>> {
        let mut query: Vec<(&str, String)> = Vec::new();
        if let Some(limit) = params.limit {
            query.push(("limit", limit.to_string()));
        }
        if let Some(ref before) = params.before_id {
            query.push(("before_id", before.clone()));
        }
        if let Some(ref after) = params.after_id {
            query.push(("after_id", after.clone()));
        }
        self.get_json(&["v1", "models"], &query).await
    }

    // ── Beta: Files ───────────────────────────────────────────────────

    #[cfg(feature = "beta")]
    pub async fn files_upload(
        &self,
        file_data: bytes::Bytes,
        filename: &str,
        purpose: &str,
    ) -> Result<crate::beta::files::FileMetadata> {
        let form = reqwest::multipart::Form::new()
            .part(
                "file",
                reqwest::multipart::Part::bytes(file_data.to_vec()).file_name(filename.to_owned()),
            )
            .text("purpose", purpose.to_owned());

        let mut req = BackendRequest {
            method: reqwest::Method::POST,
            path_segments: vec!["v1".into(), "files".into()],
            query_params: vec![],
            headers: HeaderMap::new(),
            body: None,
        };

        req = self.backend.prepare_request(req)?;
        let url = self.resolve_backend_url(&req)?;
        req = self.backend.authorize_request(req, &url)?;

        // Remove content-type — multipart sets its own
        req.headers.remove("content-type");

        let response = self
            .http
            .post(url.as_str())
            .headers(req.headers)
            .multipart(form)
            .send()
            .await
            .map_err(AnthropicError::Http)?;

        self.handle_response(response).await
    }

    #[cfg(feature = "beta")]
    pub async fn files_download(&self, file_id: &str) -> Result<bytes::Bytes> {
        let mut req = BackendRequest {
            method: reqwest::Method::GET,
            path_segments: vec!["v1".into(), "files".into(), file_id.into(), "content".into()],
            query_params: vec![],
            headers: HeaderMap::new(),
            body: None,
        };

        req = self.backend.prepare_request(req)?;
        let url = self.resolve_backend_url(&req)?;
        req = self.backend.authorize_request(req, &url)?;

        let response = self
            .http
            .get(url.as_str())
            .headers(req.headers)
            .send()
            .await
            .map_err(AnthropicError::Http)?;

        let status = response.status().as_u16();
        if status != 200 {
            let hdrs = response.headers().clone();
            let body = response.text().await.unwrap_or_default();
            return Err(AnthropicError::from_status(status, hdrs, body));
        }

        response.bytes().await.map_err(AnthropicError::Http)
    }

    #[cfg(feature = "beta")]
    pub async fn files_list(
        &self,
        params: crate::beta::files::FileListParams,
    ) -> Result<Page<crate::beta::files::FileMetadata>> {
        let mut query: Vec<(&str, String)> = Vec::new();
        if let Some(limit) = params.limit {
            query.push(("limit", limit.to_string()));
        }
        if let Some(ref after) = params.after_id {
            query.push(("after_id", after.clone()));
        }
        self.get_json(&["v1", "files"], &query).await
    }

    #[cfg(feature = "beta")]
    pub async fn files_delete(
        &self,
        file_id: &str,
    ) -> Result<crate::beta::files::DeletedFile> {
        self.delete_json(&["v1", "files", file_id]).await
    }

    // ── Beta: Skills ──────────────────────────────────────────────────

    /// Create a skill.
    #[cfg(feature = "beta")]
    pub async fn skills_create(
        &self,
        params: crate::beta::skills::SkillCreateParams,
    ) -> Result<crate::beta::skills::SkillResponse> {
        let body = serde_json::to_value(&params).map_err(AnthropicError::Serialization)?;
        self.post_json(&["v1", "skills"], body).await
    }

    /// Retrieve a skill by ID.
    #[cfg(feature = "beta")]
    pub async fn skills_retrieve(
        &self,
        skill_id: &str,
    ) -> Result<crate::beta::skills::SkillResponse> {
        self.get_json(&["v1", "skills", skill_id], &[]).await
    }

    /// List skills.
    #[cfg(feature = "beta")]
    pub async fn skills_list(
        &self,
        params: crate::beta::skills::SkillListParams,
    ) -> Result<Page<crate::beta::skills::SkillResponse>> {
        let mut query: Vec<(&str, String)> = Vec::new();
        if let Some(limit) = params.limit {
            query.push(("limit", limit.to_string()));
        }
        if let Some(ref after) = params.after_id {
            query.push(("after_id", after.clone()));
        }
        self.get_json(&["v1", "skills"], &query).await
    }

    /// Delete a skill.
    #[cfg(feature = "beta")]
    pub async fn skills_delete(
        &self,
        skill_id: &str,
    ) -> Result<crate::beta::skills::DeletedSkill> {
        self.delete_json(&["v1", "skills", skill_id]).await
    }

    /// List versions of a skill.
    #[cfg(feature = "beta")]
    pub async fn skill_versions_list(
        &self,
        skill_id: &str,
        params: crate::beta::skills::SkillVersionListParams,
    ) -> Result<Page<crate::beta::skills::SkillVersionResponse>> {
        let mut query: Vec<(&str, String)> = Vec::new();
        if let Some(limit) = params.limit {
            query.push(("limit", limit.to_string()));
        }
        if let Some(ref after) = params.after_id {
            query.push(("after_id", after.clone()));
        }
        self.get_json(&["v1", "skills", skill_id, "versions"], &query)
            .await
    }

    // ── Internal HTTP helpers ─────────────────────────────────────────

    async fn post_json<T: serde::de::DeserializeOwned>(
        &self,
        path: &[&str],
        body: serde_json::Value,
    ) -> Result<T> {
        self.request_with_retry(reqwest::Method::POST, path, Some(body), &[])
            .await
    }

    async fn get_json<T: serde::de::DeserializeOwned>(
        &self,
        path: &[&str],
        query: &[(&str, String)],
    ) -> Result<T> {
        self.request_with_retry(reqwest::Method::GET, path, None, query)
            .await
    }

    async fn delete_json<T: serde::de::DeserializeOwned>(&self, path: &[&str]) -> Result<T> {
        self.request_with_retry(reqwest::Method::DELETE, path, None, &[])
            .await
    }

    /// Resolve the URL for a BackendRequest using the backend's base URL.
    fn resolve_backend_url(&self, req: &BackendRequest) -> Result<url::Url> {
        req.resolve_url(self.backend.base_url())
    }

    async fn request_with_retry<T: serde::de::DeserializeOwned>(
        &self,
        method: reqwest::Method,
        path: &[&str],
        body: Option<serde_json::Value>,
        query: &[(&str, String)],
    ) -> Result<T> {
        let idempotency_key = uuid::Uuid::new_v4().to_string();
        let mut last_error: Option<AnthropicError> = None;

        for attempt in 0..=self.retry_config.max_retries {
            // Build BackendRequest
            let mut req = BackendRequest {
                method: method.clone(),
                path_segments: path.iter().map(|s| s.to_string()).collect(),
                query_params: query.iter().map(|(k, v)| (k.to_string(), v.clone())).collect(),
                headers: HeaderMap::new(),
                body: body.clone(),
            };

            // Let backend prepare the request
            req = self.backend.prepare_request(req)?;

            // Resolve URL
            let url = self.resolve_backend_url(&req)?;

            // Let backend authorize the request
            req = self.backend.authorize_request(req, &url)?;

            // Add client-level headers
            if let Ok(val) = HeaderValue::from_str(&idempotency_key) {
                req.headers.insert("idempotency-key", val);
            }
            if attempt > 0 {
                if let Ok(val) = HeaderValue::from_str(&attempt.to_string()) {
                    req.headers.insert("x-stainless-retry-count", val);
                }
            }

            // Build and send reqwest request
            let mut http_req = self
                .http
                .request(req.method, url.as_str())
                .headers(req.headers);
            if !req.query_params.is_empty() {
                let qp: Vec<(String, String)> = req.query_params;
                http_req = http_req.query(&qp);
            }
            if let Some(ref b) = req.body {
                http_req = http_req.json(b);
            }

            let result = http_req.send().await;

            match result {
                Ok(response) => {
                    let status = response.status().as_u16();
                    if (200..300).contains(&status) {
                        return self.handle_response(response).await;
                    }

                    let resp_headers = response.headers().clone();
                    let body_text = response.text().await.unwrap_or_default();
                    let err =
                        AnthropicError::from_status(status, resp_headers.clone(), body_text);

                    if attempt < self.retry_config.max_retries && is_retryable_status(status) {
                        let backoff = parse_retry_after(&resp_headers, &self.retry_config)
                            .unwrap_or_else(|| {
                                calculate_backoff(attempt + 1, &self.retry_config)
                            });
                        debug!(attempt, ?backoff, status, "retrying request");
                        tokio::time::sleep(backoff).await;
                        last_error = Some(err);
                        continue;
                    }

                    return Err(err);
                }
                Err(e) => {
                    let err = AnthropicError::Http(e);
                    if attempt < self.retry_config.max_retries && err.is_retryable() {
                        let backoff = calculate_backoff(attempt + 1, &self.retry_config);
                        warn!(attempt, ?backoff, "retrying after HTTP error");
                        tokio::time::sleep(backoff).await;
                        last_error = Some(err);
                        continue;
                    }
                    return Err(err);
                }
            }
        }

        Err(last_error.unwrap_or_else(|| AnthropicError::Config("retry loop exhausted".into())))
    }

    async fn handle_response<T: serde::de::DeserializeOwned>(
        &self,
        response: reqwest::Response,
    ) -> Result<T> {
        let status = response.status().as_u16();
        if (200..300).contains(&status) {
            let text = response.text().await.map_err(AnthropicError::Http)?;
            serde_json::from_str(&text).map_err(|e| {
                AnthropicError::InvalidData(format!(
                    "failed to deserialize response: {e}\nbody: {text}"
                ))
            })
        } else {
            let headers = response.headers().clone();
            let body = response.text().await.unwrap_or_default();
            Err(AnthropicError::from_status(status, headers, body))
        }
    }
}

/// Convert a byte stream (JSONL) into a stream of parsed items.
fn jsonl_stream(
    byte_stream: impl Stream<Item = std::result::Result<Bytes, reqwest::Error>> + Send + 'static,
) -> impl Stream<Item = Result<MessageBatchIndividualResponse>> + Send {
    async_stream(Box::pin(byte_stream))
}

fn async_stream(
    mut stream: Pin<Box<dyn Stream<Item = std::result::Result<Bytes, reqwest::Error>> + Send>>,
) -> impl Stream<Item = Result<MessageBatchIndividualResponse>> + Send {
    use tokio_stream::StreamExt;

    let (tx, rx) = tokio::sync::mpsc::channel(32);

    tokio::spawn(async move {
        let mut buf = String::new();

        while let Some(chunk) = StreamExt::next(&mut stream).await {
            match chunk {
                Ok(bytes) => {
                    buf.push_str(&String::from_utf8_lossy(&bytes));
                    while let Some(newline_pos) = buf.find('\n') {
                        let line = buf[..newline_pos].trim().to_owned();
                        buf = buf[newline_pos + 1..].to_owned();
                        if line.is_empty() {
                            continue;
                        }
                        let item =
                            serde_json::from_str(&line).map_err(AnthropicError::Serialization);
                        if tx.send(item).await.is_err() {
                            return;
                        }
                    }
                }
                Err(e) => {
                    let _ = tx.send(Err(AnthropicError::Http(e))).await;
                    return;
                }
            }
        }

        let remaining = buf.trim().to_owned();
        if !remaining.is_empty() {
            let item =
                serde_json::from_str(&remaining).map_err(AnthropicError::Serialization);
            let _ = tx.send(item).await;
        }
    });

    tokio_stream::wrappers::ReceiverStream::new(rx)
}
