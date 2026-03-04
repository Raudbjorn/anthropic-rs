use serde::{Deserialize, Serialize};

use crate::types::message::Message;

/// An individual response within a batch.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageBatchIndividualResponse {
    pub custom_id: String,
    pub result: MessageBatchResult,
}

/// The result of a batch request — success, error, expired, or canceled.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MessageBatchResult {
    #[serde(rename = "succeeded")]
    Succeeded { message: Box<Message> },

    #[serde(rename = "errored")]
    Errored { error: BatchError },

    #[serde(rename = "expired")]
    Expired,

    #[serde(rename = "canceled")]
    Canceled,
}

/// Error details for a failed batch request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchError {
    #[serde(rename = "type")]
    pub error_type: String,
    pub message: String,
}
