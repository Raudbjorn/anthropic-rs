use serde::Serialize;

/// Parameters for listing models.
#[derive(Debug, Clone, Default, Serialize)]
pub struct ModelListParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after_id: Option<String>,
}
