use serde::{Deserialize, Serialize};

use super::cache_control::CacheControl;

/// A document content block param (PDF, plain text, etc.).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentBlockParam {
    pub source: DocumentSource,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub citations: Option<CitationsConfig>,
}

/// Document source — base64-encoded, URL, plain text, or file reference.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum DocumentSource {
    #[serde(rename = "base64")]
    Base64 {
        media_type: DocumentMediaType,
        data: String,
    },
    #[serde(rename = "url")]
    Url {
        url: String,
    },
    #[serde(rename = "text")]
    Text {
        text: String,
    },
    #[serde(rename = "file")]
    File {
        file_id: String,
    },
}

/// Supported document media types.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DocumentMediaType {
    #[serde(rename = "application/pdf")]
    Pdf,
    #[serde(rename = "text/plain")]
    PlainText,
    #[serde(rename = "text/html")]
    Html,
    #[serde(rename = "text/csv")]
    Csv,
}

/// Configuration for document citations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CitationsConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
}
