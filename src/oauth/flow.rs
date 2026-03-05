//! OAuth 2.0 authorization code flow with PKCE.

use tracing::{debug, info, warn};
use url::Url;

use super::config::OAuthConfig;
use super::error::{OAuthError, Result};
use super::pkce::Pkce;
use super::storage::TokenStorage;
use super::token::{RefreshRequest, TokenInfo, TokenRequest, TokenResponse};

/// State for an in-progress OAuth flow.
#[derive(Debug)]
pub struct OAuthFlowState {
    /// PKCE verifier (needed for token exchange).
    pub pkce: Pkce,
    /// State parameter for CSRF protection.
    pub state: String,
}

/// OAuth flow orchestrator.
pub struct OAuthFlow<S: TokenStorage> {
    config: OAuthConfig,
    storage: S,
    http_client: reqwest::Client,
    flow_state: Option<OAuthFlowState>,
}

impl<S: TokenStorage> OAuthFlow<S> {
    /// Create a new OAuth flow with default config.
    pub fn new(storage: S) -> Self {
        Self::with_config(storage, OAuthConfig::default())
    }

    /// Create a new OAuth flow with custom config.
    pub fn with_config(storage: S, config: OAuthConfig) -> Self {
        Self {
            config,
            storage,
            http_client: reqwest::Client::new(),
            flow_state: None,
        }
    }

    /// Get a reference to the config.
    pub fn config(&self) -> &OAuthConfig {
        &self.config
    }

    /// Get a reference to the storage.
    pub fn storage(&self) -> &S {
        &self.storage
    }

    /// Check if a valid token exists.
    pub fn is_authenticated(&self) -> Result<bool> {
        match self.storage.load()? {
            Some(token) => {
                if token.is_expired() {
                    debug!("Token exists but is expired");
                    Ok(false)
                } else {
                    debug!("Valid token exists");
                    Ok(true)
                }
            }
            None => {
                debug!("No token stored");
                Ok(false)
            }
        }
    }

    /// Start the OAuth authorization flow.
    ///
    /// Returns the authorization URL and flow state. The user should open
    /// the URL in their browser, then call `exchange_code` with the received
    /// authorization code and state.
    pub fn start_authorization(&mut self) -> Result<(String, OAuthFlowState)> {
        let pkce = Pkce::generate();
        // State is a separate random value for CSRF protection (not the PKCE verifier).
        let state = {
            use base64::Engine;
            let mut buf = [0u8; 32];
            rand::Rng::fill(&mut rand::thread_rng(), &mut buf);
            base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(buf)
        };

        let mut url = Url::parse(&self.config.authorize_url)?;
        url.query_pairs_mut()
            .append_pair("response_type", "code")
            .append_pair("client_id", &self.config.client_id)
            .append_pair("redirect_uri", &self.config.redirect_uri)
            .append_pair("scope", &self.config.scopes.join(" "))
            .append_pair("code_challenge", &pkce.challenge)
            .append_pair("code_challenge_method", pkce.method)
            .append_pair("state", &state);

        let flow_state = OAuthFlowState {
            pkce: pkce.clone(),
            state: state.clone(),
        };

        self.flow_state = Some(OAuthFlowState {
            pkce,
            state,
        });

        info!(url = %url, "Started OAuth authorization flow");
        Ok((url.to_string(), flow_state))
    }

    /// Exchange an authorization code for tokens.
    pub async fn exchange_code(&mut self, code: &str, state: Option<&str>) -> Result<TokenInfo> {
        let flow_state = self
            .flow_state
            .take()
            .ok_or_else(|| OAuthError::OAuth("no active OAuth flow - call start_authorization first".into()))?;

        // Verify state
        let received_state = state.ok_or_else(|| OAuthError::InvalidState {
            expected: flow_state.state.clone(),
            actual: "missing".to_string(),
        })?;

        if received_state != flow_state.state {
            return Err(OAuthError::InvalidState {
                expected: flow_state.state,
                actual: received_state.to_string(),
            });
        }

        let response = self
            .http_client
            .post(&self.config.token_url)
            .form(&TokenRequest {
                code,
                grant_type: "authorization_code",
                client_id: &self.config.client_id,
                redirect_uri: &self.config.redirect_uri,
                code_verifier: &flow_state.pkce.verifier,
                state,
            })
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body = response.text().await.unwrap_or_default();
            warn!(status, body = %body, "Token exchange failed");
            return Err(OAuthError::OAuth(format!(
                "token exchange failed ({status}): {body}"
            )));
        }

        let token_response: TokenResponse = response.json().await?;
        let refresh_token = token_response.refresh_token.ok_or_else(|| {
            OAuthError::OAuth("token exchange response missing refresh_token".into())
        })?;
        let token = TokenInfo::new(
            token_response.access_token,
            refresh_token,
            token_response.expires_in,
        );

        self.storage.save(&token)?;
        info!("Token exchange successful, token saved");

        Ok(token)
    }

    /// Refresh an existing token.
    ///
    /// Retries up to 3 times before giving up. On repeated failure the stored
    /// token is cleared so the user is forced to re-authenticate.
    pub async fn refresh_token(&self) -> Result<TokenInfo> {
        const MAX_ATTEMPTS: u32 = 3;

        let current_token = self
            .storage
            .load()?
            .ok_or(OAuthError::NotAuthenticated)?;

        let mut last_err = None;

        for attempt in 1..=MAX_ATTEMPTS {
            debug!(attempt, "Attempting token refresh");

            let result = self
                .http_client
                .post(&self.config.token_url)
                .form(&RefreshRequest {
                    grant_type: "refresh_token",
                    refresh_token: &current_token.refresh_token,
                    client_id: &self.config.client_id,
                })
                .send()
                .await;

            match result {
                Ok(response) if response.status().is_success() => {
                    let token_response: TokenResponse = response.json().await?;
                    // Fall back to existing refresh token if the response omits it.
                    let refresh_token = token_response
                        .refresh_token
                        .unwrap_or_else(|| current_token.refresh_token.clone());
                    let token = TokenInfo::new(
                        token_response.access_token,
                        refresh_token,
                        token_response.expires_in,
                    );
                    self.storage.save(&token)?;
                    info!("Token refresh successful on attempt {attempt}");
                    return Ok(token);
                }
                Ok(response) => {
                    let status = response.status().as_u16();
                    let body = response.text().await.unwrap_or_default();
                    warn!(attempt, status, body = %body, "Token refresh attempt failed");
                    last_err = Some(format!("({status}): {body}"));
                }
                Err(e) => {
                    warn!(attempt, error = %e, "Token refresh request failed");
                    last_err = Some(e.to_string());
                }
            }
        }

        // All attempts exhausted — clear the stored token so the user
        // is forced to re-authenticate (matches browser extension behavior).
        warn!("All {MAX_ATTEMPTS} refresh attempts failed, clearing stored token");
        if let Err(e) = self.storage.remove() {
            warn!(error = %e, "Failed to clear stored token after refresh failure");
        }

        Err(OAuthError::RefreshFailed(format!(
            "token refresh failed after {MAX_ATTEMPTS} attempts: {}",
            last_err.unwrap_or_default()
        )))
    }

    /// Get a valid access token, refreshing if necessary.
    pub async fn get_access_token(&self) -> Result<String> {
        let token = self
            .storage
            .load()?
            .ok_or(OAuthError::NotAuthenticated)?;

        if token.needs_refresh() {
            debug!("Token needs refresh");
            let refreshed = self.refresh_token().await?;
            Ok(refreshed.access_token)
        } else {
            Ok(token.access_token)
        }
    }

    /// Log out by removing the stored token.
    pub fn logout(&self) -> Result<()> {
        self.storage.remove()?;
        info!("Logged out, token removed");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::oauth::storage::MemoryTokenStorage;

    #[test]
    fn test_start_authorization_url_params() {
        let storage = MemoryTokenStorage::new();
        let mut flow = OAuthFlow::new(storage);

        let (url, state) = flow.start_authorization().unwrap();

        assert!(url.contains("response_type=code"));
        assert!(url.contains("client_id="));
        assert!(url.contains("code_challenge="));
        assert!(url.contains("code_challenge_method=S256"));
        assert!(url.contains("redirect_uri="));
        assert!(url.contains("scope="));
        assert!(!state.state.is_empty());
        assert!(!state.pkce.verifier.is_empty());
    }

    #[test]
    fn test_state_is_separate_from_verifier() {
        let storage = MemoryTokenStorage::new();
        let mut flow = OAuthFlow::new(storage);

        let (_url, state) = flow.start_authorization().unwrap();

        // State is a separate random value, not the PKCE verifier
        assert_ne!(state.state, state.pkce.verifier);
        assert!(!state.state.is_empty());
        assert_eq!(state.pkce.verifier.len(), 43);
    }

    #[test]
    fn test_is_authenticated_no_token() {
        let storage = MemoryTokenStorage::new();
        let flow = OAuthFlow::new(storage);
        assert!(!flow.is_authenticated().unwrap());
    }

    #[test]
    fn test_is_authenticated_with_valid_token() {
        let token = TokenInfo::new("access".into(), "refresh".into(), 3600);
        let storage = MemoryTokenStorage::with_token(token);
        let flow = OAuthFlow::new(storage);
        assert!(flow.is_authenticated().unwrap());
    }

    #[test]
    fn test_is_authenticated_with_expired_token() {
        let token = TokenInfo::with_expires_at("access".into(), "refresh".into(), 0);
        let storage = MemoryTokenStorage::with_token(token);
        let flow = OAuthFlow::new(storage);
        assert!(!flow.is_authenticated().unwrap());
    }

    #[test]
    fn test_logout() {
        let token = TokenInfo::new("access".into(), "refresh".into(), 3600);
        let storage = MemoryTokenStorage::with_token(token);
        let flow = OAuthFlow::new(storage);

        assert!(flow.is_authenticated().unwrap());
        flow.logout().unwrap();
        assert!(!flow.is_authenticated().unwrap());
    }

    #[test]
    fn test_custom_config() {
        let config = OAuthConfig::new().with_client_id("custom-id");
        let storage = MemoryTokenStorage::new();
        let mut flow = OAuthFlow::with_config(storage, config);

        let (url, _) = flow.start_authorization().unwrap();
        assert!(url.contains("client_id=custom-id"));
    }
}
