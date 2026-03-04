//! OAuth token types.

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Safety margin for token expiry checks (60 seconds).
const EXPIRY_SAFETY_MARGIN_SECS: i64 = 60;

/// Proactive refresh buffer (5 minutes).
const REFRESH_BUFFER_SECS: i64 = 300;

/// OAuth token information.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TokenInfo {
    /// Token type (typically "Bearer").
    pub token_type: String,
    /// OAuth access token for API requests.
    pub access_token: String,
    /// OAuth refresh token.
    pub refresh_token: String,
    /// Unix timestamp when the access token expires.
    pub expires_at: i64,
}

impl TokenInfo {
    /// Create a new TokenInfo from token exchange response.
    pub fn new(access_token: String, refresh_token: String, expires_in: i64) -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            token_type: "Bearer".to_string(),
            access_token,
            refresh_token,
            expires_at: now + expires_in,
        }
    }

    /// Create a TokenInfo with a specific expiration timestamp.
    pub fn with_expires_at(access_token: String, refresh_token: String, expires_at: i64) -> Self {
        Self {
            token_type: "Bearer".to_string(),
            access_token,
            refresh_token,
            expires_at,
        }
    }

    /// Check if the token is expired (with safety margin).
    #[must_use]
    pub fn is_expired(&self) -> bool {
        let now = chrono::Utc::now().timestamp();
        self.expires_at <= now + EXPIRY_SAFETY_MARGIN_SECS
    }

    /// Check if the token should be proactively refreshed (within 5-min buffer).
    #[must_use]
    pub fn needs_refresh(&self) -> bool {
        let now = chrono::Utc::now().timestamp();
        self.expires_at <= now + REFRESH_BUFFER_SECS
    }

    /// Get the duration until the access token expires.
    pub fn time_until_expiry(&self) -> Duration {
        let now = chrono::Utc::now().timestamp();
        let remaining = self.expires_at - now;
        if remaining > 0 {
            Duration::from_secs(remaining as u64)
        } else {
            Duration::ZERO
        }
    }

    /// Get the expiry as a chrono DateTime.
    pub fn expires_at_datetime(&self) -> chrono::DateTime<chrono::Utc> {
        chrono::DateTime::from_timestamp(self.expires_at, 0)
            .unwrap_or_else(chrono::Utc::now)
    }
}

/// Token request payload (form-encoded per RFC 6749 / Anthropic's token endpoint).
#[derive(Serialize)]
pub(crate) struct TokenRequest<'a> {
    pub code: &'a str,
    pub grant_type: &'a str,
    pub client_id: &'a str,
    pub redirect_uri: &'a str,
    pub code_verifier: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<&'a str>,
}

/// Refresh request payload.
#[derive(Serialize)]
pub(crate) struct RefreshRequest<'a> {
    pub grant_type: &'a str,
    pub refresh_token: &'a str,
    pub client_id: &'a str,
}

/// Token response from OAuth endpoint.
///
/// `refresh_token` may be absent in refresh-grant responses, in which case
/// the caller should keep using the existing refresh token.
#[derive(Debug, Deserialize)]
pub(crate) struct TokenResponse {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: i64,
    #[serde(rename = "token_type")]
    #[allow(dead_code)]
    pub token_type: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let token = TokenInfo::new("access".into(), "refresh".into(), 3600);
        assert_eq!(token.token_type, "Bearer");
        assert_eq!(token.access_token, "access");
        assert_eq!(token.refresh_token, "refresh");
        assert!(!token.is_expired());
    }

    #[test]
    fn test_is_expired() {
        let expired = TokenInfo::with_expires_at("access".into(), "refresh".into(), 0);
        assert!(expired.is_expired());

        // Within safety margin
        let soon = TokenInfo::with_expires_at(
            "access".into(),
            "refresh".into(),
            chrono::Utc::now().timestamp() + 30,
        );
        assert!(soon.is_expired());

        let fresh = TokenInfo::new("access".into(), "refresh".into(), 3600);
        assert!(!fresh.is_expired());
    }

    #[test]
    fn test_needs_refresh() {
        let fresh = TokenInfo::new("access".into(), "refresh".into(), 3600);
        assert!(!fresh.needs_refresh());

        // 4 minutes = within 5-min buffer
        let soon = TokenInfo::with_expires_at(
            "access".into(),
            "refresh".into(),
            chrono::Utc::now().timestamp() + 240,
        );
        assert!(soon.needs_refresh());

        // 6 minutes = outside buffer
        let later = TokenInfo::with_expires_at(
            "access".into(),
            "refresh".into(),
            chrono::Utc::now().timestamp() + 360,
        );
        assert!(!later.needs_refresh());

        let expired = TokenInfo::with_expires_at("access".into(), "refresh".into(), 0);
        assert!(expired.needs_refresh());
    }

    #[test]
    fn test_time_until_expiry() {
        let token = TokenInfo::new("access".into(), "refresh".into(), 3600);
        let remaining = token.time_until_expiry();
        assert!(remaining.as_secs() >= 3595);
        assert!(remaining.as_secs() <= 3600);

        let expired = TokenInfo::with_expires_at("access".into(), "refresh".into(), 0);
        assert_eq!(expired.time_until_expiry(), Duration::ZERO);
    }

    #[test]
    fn test_serde_roundtrip() {
        let token = TokenInfo::new("access".into(), "refresh".into(), 3600);
        let json = serde_json::to_string(&token).unwrap();
        let restored: TokenInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(token, restored);
    }

    #[test]
    fn test_expires_at_datetime() {
        let token = TokenInfo::new("access".into(), "refresh".into(), 3600);
        let dt = token.expires_at_datetime();
        assert!(dt > chrono::Utc::now());
    }
}
