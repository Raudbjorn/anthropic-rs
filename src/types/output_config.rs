use serde::{Deserialize, Serialize};

/// Output configuration for structured output and effort control.
///
/// This is a struct with optional fields (not a tagged enum). The API accepts
/// any combination of `effort` and `format`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    /// Effort level for thinking budget.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effort: Option<Effort>,
    /// Output format constraint (JSON schema or text).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub format: Option<JsonOutputFormat>,
}

impl OutputConfig {
    /// Create an output config with just an effort level.
    pub fn with_effort(effort: Effort) -> Self {
        Self {
            effort: Some(effort),
            format: None,
        }
    }

    /// Create an output config with a JSON schema format.
    pub fn json_schema(schema: serde_json::Value) -> Self {
        Self {
            effort: None,
            format: Some(JsonOutputFormat::JsonSchema { schema: Some(schema) }),
        }
    }

    /// Create an output config with text format.
    pub fn text() -> Self {
        Self {
            effort: None,
            format: Some(JsonOutputFormat::Text),
        }
    }
}

/// Budget effort level for thinking.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Effort {
    Low,
    Medium,
    High,
    Max,
}

/// Output format constraint.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum JsonOutputFormat {
    #[serde(rename = "json")]
    JsonSchema {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        schema: Option<serde_json::Value>,
    },
    #[serde(rename = "text")]
    Text,
}
