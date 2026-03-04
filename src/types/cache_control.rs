use serde::{Deserialize, Serialize};

/// Cache control directive for prompt caching.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum CacheControl {
    #[serde(rename = "ephemeral")]
    Ephemeral {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        ttl: Option<CacheTtl>,
    },
}

impl CacheControl {
    pub fn ephemeral() -> Self {
        Self::Ephemeral { ttl: None }
    }

    pub fn ephemeral_with_ttl(ttl: CacheTtl) -> Self {
        Self::Ephemeral { ttl: Some(ttl) }
    }
}

/// Time-to-live values for cache control.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum CacheTtl {
    #[serde(rename = "5m")]
    FiveMinutes,
    #[serde(rename = "1h")]
    OneHour,
}
