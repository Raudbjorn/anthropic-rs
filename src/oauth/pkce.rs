//! PKCE (Proof Key for Code Exchange) S256 generation and verification.

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use rand::Rng;
use sha2::{Digest, Sha256};

/// PKCE challenge method.
const PKCE_METHOD: &str = "S256";

/// PKCE verifier length in bytes (produces 43-char base64url string).
const PKCE_VERIFIER_LENGTH: usize = 32;

/// PKCE (Proof Key for Code Exchange) data.
#[derive(Debug, Clone)]
pub struct Pkce {
    /// The code verifier (sent during token exchange).
    pub verifier: String,
    /// The code challenge (sent during authorization).
    pub challenge: String,
    /// The challenge method (always "S256").
    pub method: &'static str,
}

impl Pkce {
    /// Generate a new PKCE challenge/verifier pair.
    #[must_use]
    pub fn generate() -> Self {
        let mut rng = rand::thread_rng();
        let random_bytes: [u8; PKCE_VERIFIER_LENGTH] = rng.gen();
        let verifier = URL_SAFE_NO_PAD.encode(random_bytes);

        let mut hasher = Sha256::new();
        hasher.update(verifier.as_bytes());
        let hash = hasher.finalize();
        let challenge = URL_SAFE_NO_PAD.encode(hash);

        Self {
            verifier,
            challenge,
            method: PKCE_METHOD,
        }
    }

    /// Verify that a challenge matches a verifier.
    #[must_use]
    pub fn verify(verifier: &str, challenge: &str) -> bool {
        let mut hasher = Sha256::new();
        hasher.update(verifier.as_bytes());
        let hash = hasher.finalize();
        let expected = URL_SAFE_NO_PAD.encode(hash);
        expected == challenge
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generation() {
        let pkce = Pkce::generate();
        assert!(!pkce.verifier.is_empty());
        assert!(!pkce.challenge.is_empty());
        assert_eq!(pkce.method, "S256");
        assert!(Pkce::verify(&pkce.verifier, &pkce.challenge));
    }

    #[test]
    fn test_verification() {
        let pkce = Pkce::generate();
        assert!(Pkce::verify(&pkce.verifier, &pkce.challenge));
    }

    #[test]
    fn test_wrong_verifier() {
        let pkce = Pkce::generate();
        assert!(!Pkce::verify("wrong_verifier", &pkce.challenge));
    }

    #[test]
    fn test_verifier_length() {
        let pkce = Pkce::generate();
        // 32 bytes base64url-encoded (no padding) = 43 chars
        assert_eq!(pkce.verifier.len(), 43);
    }

    #[test]
    fn test_uniqueness() {
        let pkce1 = Pkce::generate();
        let pkce2 = Pkce::generate();
        assert_ne!(pkce1.verifier, pkce2.verifier);
        assert_ne!(pkce1.challenge, pkce2.challenge);
    }
}
