use serde::{Deserialize, Serialize};

use super::cache_control::CacheControl;
use super::citation::TextCitation;

/// A text content block in a response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextBlock {
    pub text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub citations: Option<Vec<TextCitation>>,
}

/// A text content block param for input.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextBlockParam {
    pub text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub citations: Option<Vec<TextCitation>>,
}

/// Delta for streaming text updates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextDelta {
    pub text: String,
}
