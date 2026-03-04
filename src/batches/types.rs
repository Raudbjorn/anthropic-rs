use serde::{Deserialize, Serialize};

/// A message batch.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageBatch {
    pub id: String,
    #[serde(rename = "type")]
    pub batch_type: String,
    pub processing_status: ProcessingStatus,
    pub request_counts: RequestCounts,
    pub created_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ended_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cancel_initiated_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub archived_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub results_url: Option<String>,
}

/// Processing status of a batch.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProcessingStatus {
    InProgress,
    Canceling,
    Ended,
}

/// Request counts within a batch.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RequestCounts {
    pub processing: u64,
    pub succeeded: u64,
    pub errored: u64,
    pub canceled: u64,
    pub expired: u64,
}

/// Response when deleting a batch.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeletedMessageBatch {
    pub id: String,
    #[serde(rename = "type")]
    pub deleted_type: String,
}
