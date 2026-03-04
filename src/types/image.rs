use serde::{Deserialize, Serialize};

use super::cache_control::CacheControl;

/// An image content block param.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageBlockParam {
    pub source: ImageSource,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
}

/// Image source — base64-encoded, URL, or file reference.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ImageSource {
    #[serde(rename = "base64")]
    Base64 {
        media_type: MediaType,
        data: String,
    },
    #[serde(rename = "url")]
    Url {
        url: String,
    },
    #[serde(rename = "file")]
    File {
        file_id: String,
    },
}

/// Supported image media types.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MediaType {
    #[serde(rename = "image/jpeg")]
    Jpeg,
    #[serde(rename = "image/png")]
    Png,
    #[serde(rename = "image/gif")]
    Gif,
    #[serde(rename = "image/webp")]
    Webp,
}
