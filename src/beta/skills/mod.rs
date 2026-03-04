use serde::{Deserialize, Serialize};

/// A skill response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillResponse {
    pub id: String,
    #[serde(rename = "type")]
    pub skill_type: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub created_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
}

/// A skill version response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillVersionResponse {
    pub id: String,
    pub skill_id: String,
    pub version: u64,
    pub created_at: String,
}

/// Parameters for listing skills.
#[derive(Debug, Clone, Default)]
pub struct SkillListParams {
    pub limit: Option<u64>,
    pub after_id: Option<String>,
}

/// Parameters for creating a skill.
#[derive(Debug, Clone, Serialize)]
pub struct SkillCreateParams {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Parameters for listing skill versions.
#[derive(Debug, Clone, Default)]
pub struct SkillVersionListParams {
    pub limit: Option<u64>,
    pub after_id: Option<String>,
}

/// Response when deleting a skill.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeletedSkill {
    pub id: String,
    #[serde(rename = "type")]
    pub deleted_type: String,
}
