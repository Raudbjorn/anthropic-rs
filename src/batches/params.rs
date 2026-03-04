use serde::Serialize;

use crate::messages::create::MessageCreateParams;

/// Parameters for creating a batch.
#[derive(Debug, Clone, Serialize)]
pub struct BatchCreateParams {
    pub requests: Vec<BatchRequest>,
}

/// A single request within a batch.
#[derive(Debug, Clone, Serialize)]
pub struct BatchRequest {
    pub custom_id: String,
    pub params: MessageCreateParams,
}

/// Parameters for listing batches.
#[derive(Debug, Clone, Default, Serialize)]
pub struct BatchListParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after_id: Option<String>,
}
