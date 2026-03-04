use serde::{Deserialize, Serialize};

/// Information about an available model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    #[serde(rename = "type")]
    pub model_type: String,
    pub display_name: String,
    pub created_at: String,
}
