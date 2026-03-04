use serde::{Deserialize, Serialize};

/// Container information for code execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Container {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<String>,
}

/// Container upload block in a response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerUploadBlock {
    pub file_id: String,
}
