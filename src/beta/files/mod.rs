use serde::{Deserialize, Serialize};

/// Metadata for an uploaded file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    pub id: String,
    #[serde(rename = "type")]
    pub file_type: String,
    pub filename: String,
    pub purpose: String,
    pub size_bytes: u64,
    pub created_at: String,
}

/// Response when deleting a file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeletedFile {
    pub id: String,
    #[serde(rename = "type")]
    pub deleted_type: String,
}

/// Parameters for listing files.
#[derive(Debug, Clone, Default)]
pub struct FileListParams {
    pub limit: Option<u64>,
    pub after_id: Option<String>,
}
