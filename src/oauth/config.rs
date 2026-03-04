//! OAuth configuration and endpoint constants.

/// Default OAuth client ID (public by design, security via PKCE).
///
/// Shared across first-party Anthropic tools (Claude Code, opencode, etc.).
pub const DEFAULT_CLIENT_ID: &str = "9d1c250a-e61b-44d9-88ed-5944d1962f5e";

/// Default authorization URL.
///
/// `claude.ai/oauth/authorize` — authorizes via the Claude Pro/Max flow.
/// For console-only auth, use `console.anthropic.com/oauth/authorize`.
pub const DEFAULT_AUTHORIZE_URL: &str = "https://claude.ai/oauth/authorize";

/// Default token URL.
///
/// Confirmed endpoint for both authorization-code and refresh-token grants.
/// Note: the Chrome extension uses `platform.claude.com` which is a different
/// domain targeting browser-chat tokens; SDK/API tokens use `console.anthropic.com`.
pub const DEFAULT_TOKEN_URL: &str = "https://console.anthropic.com/v1/oauth/token";

/// Default redirect URI (Anthropic's hosted callback page).
pub const DEFAULT_REDIRECT_URI: &str = "https://console.anthropic.com/oauth/code/callback";

/// Default OAuth scopes for SDK API access.
///
/// - `org:create_api_key` — create scoped API keys on behalf of the user
/// - `user:profile` — access user profile / usage data
/// - `user:inference` — make inference requests (messages, completions)
///
/// The Chrome extension uses `user:chat` instead of `org:create_api_key` because
/// it accesses the chat interface, not the API.
pub const DEFAULT_SCOPES: &[&str] = &["org:create_api_key", "user:profile", "user:inference"];

/// OAuth configuration.
#[derive(Debug, Clone)]
pub struct OAuthConfig {
    /// OAuth client ID.
    pub client_id: String,
    /// Authorization endpoint URL.
    pub authorize_url: String,
    /// Token endpoint URL.
    pub token_url: String,
    /// Redirect URI.
    pub redirect_uri: String,
    /// Requested scopes.
    pub scopes: Vec<String>,
}

impl Default for OAuthConfig {
    fn default() -> Self {
        Self {
            client_id: DEFAULT_CLIENT_ID.to_string(),
            authorize_url: DEFAULT_AUTHORIZE_URL.to_string(),
            token_url: DEFAULT_TOKEN_URL.to_string(),
            redirect_uri: DEFAULT_REDIRECT_URI.to_string(),
            scopes: DEFAULT_SCOPES.iter().map(|&s| s.to_string()).collect(),
        }
    }
}

impl OAuthConfig {
    /// Create a new OAuth config with default values.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set a custom client ID.
    #[must_use]
    pub fn with_client_id(mut self, client_id: impl Into<String>) -> Self {
        self.client_id = client_id.into();
        self
    }

    /// Set custom scopes.
    #[must_use]
    pub fn with_scopes(mut self, scopes: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.scopes = scopes.into_iter().map(Into::into).collect();
        self
    }

    /// Set a custom redirect URI.
    #[must_use]
    pub fn with_redirect_uri(mut self, uri: impl Into<String>) -> Self {
        self.redirect_uri = uri.into();
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_values() {
        let config = OAuthConfig::default();
        assert_eq!(config.client_id, DEFAULT_CLIENT_ID);
        assert_eq!(config.authorize_url, DEFAULT_AUTHORIZE_URL);
        assert_eq!(config.token_url, DEFAULT_TOKEN_URL);
        assert_eq!(config.redirect_uri, DEFAULT_REDIRECT_URI);
        assert_eq!(config.scopes.len(), 3);
    }

    #[test]
    fn test_with_client_id() {
        let config = OAuthConfig::new().with_client_id("custom-id");
        assert_eq!(config.client_id, "custom-id");
        // Other fields unchanged
        assert_eq!(config.authorize_url, DEFAULT_AUTHORIZE_URL);
    }

    #[test]
    fn test_with_scopes() {
        let config = OAuthConfig::new().with_scopes(["scope1", "scope2"]);
        assert_eq!(config.scopes, vec!["scope1", "scope2"]);
    }

    #[test]
    fn test_with_redirect_uri() {
        let config = OAuthConfig::new().with_redirect_uri("http://localhost:8080/callback");
        assert_eq!(config.redirect_uri, "http://localhost:8080/callback");
    }
}
