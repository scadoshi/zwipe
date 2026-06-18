//! Refresh token generation and validation.
//!
//! Refresh tokens allow clients to obtain new access tokens without re-authenticating.
//! They are longer-lived than access tokens (14 days vs 24 hours) and enable persistent
//! sessions while maintaining security.
//!
//! # Security Features
//!
//! - **Single-Use**: Each refresh token can only be used once (token rotation)
//! - **SHA-256 Hashing**: Tokens are hashed before database storage
//! - **Cryptographically Random**: Generated using secure RNG (32 bytes)
//! - **Time-Limited**: 14-day expiration

use std::str::FromStr;

use chrono::{DateTime, Duration, Utc};
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

/// Refresh token lifespan (14 days).
const REFRESH_TOKEN_LIFESPAN: Duration = Duration::days(14);

// =======
//  error
// =======

/// Errors that can occur while validating a refresh token string.
#[derive(Debug, Error)]
pub enum InvalidRefreshToken {
    /// Token string is not exactly 64 characters (32 bytes hex-encoded).
    #[error("must be 64 characters")]
    Length,
    /// Token contains non-hexadecimal characters.
    #[error("must contain only hexadecimal characters")]
    InvalidCharacters,
}

// ======
//  main
// ======

/// A refresh token for obtaining new access tokens.
///
/// Generated as 32 cryptographically-random bytes, hex-encoded to 64 characters.
/// Includes expiration timestamp (14 days from creation).
///
/// # Storage
///
/// - **Database**: SHA-256 hash of `value` (not plaintext!)
/// - **Client**: Plaintext `value` sent in response body
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct RefreshToken {
    /// The token value (64 hex characters = 32 random bytes).
    pub value: String,
    /// When this token expires (14 days from creation).
    pub expires_at: DateTime<Utc>,
}

impl RefreshToken {
    /// Generates a new cryptographically-secure refresh token.
    pub fn generate() -> Self {
        let mut bytes = [0u8; 32];
        rand::rng().fill_bytes(&mut bytes);
        let value = hex::encode(bytes);
        let expires_at = Utc::now() + REFRESH_TOKEN_LIFESPAN;
        Self { value, expires_at }
    }
}

/// Trait for computing SHA-256 hashes of refresh tokens.
///
/// Used to hash tokens before database storage. The database never stores
/// plaintext refresh token values - only their SHA-256 hashes.
pub trait Sha256Hash {
    /// Computes the SHA-256 hash of this value as a hex string.
    fn sha256_hash(&self) -> String;
}

impl Sha256Hash for RefreshToken {
    fn sha256_hash(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.value.clone());
        hex::encode(hasher.finalize())
    }
}

impl Sha256Hash for String {
    fn sha256_hash(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.clone());
        hex::encode(hasher.finalize())
    }
}

/// An unvalidated refresh token string from client input.
///
/// Used during token refresh operations to validate the client-provided
/// token before looking it up in the database.
pub struct UnvalidatedRefreshToken(String);

impl std::ops::Deref for UnvalidatedRefreshToken {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromStr for UnvalidatedRefreshToken {
    type Err = InvalidRefreshToken;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if s.len() != 64 {
            return Err(InvalidRefreshToken::Length);
        }
        if !s.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(InvalidRefreshToken::InvalidCharacters);
        }
        Ok(Self(s.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_refresh_token_generate_produces_64_char_hex_value() {
        let token = RefreshToken::generate();
        assert_eq!(token.value.len(), 64);
    }

    #[test]
    fn test_refresh_token_generate_produces_unique_values() {
        let t1 = RefreshToken::generate();
        let t2 = RefreshToken::generate();
        assert_ne!(t1.value, t2.value);
    }

    #[test]
    fn test_refresh_token_generate_sets_expiry_14_days_in_future() {
        let token = RefreshToken::generate();
        let now = Utc::now();
        assert!(token.expires_at > now + Duration::days(13));
        assert!(token.expires_at < now + Duration::days(15));
    }

    #[test]
    fn test_refresh_token_sha256_hash_produces_64_char_hex() {
        let token = RefreshToken::generate();
        let hash = token.sha256_hash();
        assert_eq!(hash.len(), 64);
    }

    #[test]
    fn test_refresh_token_sha256_hash_is_deterministic() {
        let token = RefreshToken::generate();
        assert_eq!(token.sha256_hash(), token.sha256_hash());
    }

    #[test]
    fn test_refresh_token_sha256_hash_differs_from_value() {
        let token = RefreshToken::generate();
        assert_ne!(token.sha256_hash(), token.value);
    }

    #[test]
    fn test_string_sha256_hash_produces_64_char_hex() {
        let hash = "some-string".to_string().sha256_hash();
        assert_eq!(hash.len(), 64);
    }

    #[test]
    fn test_string_sha256_hash_is_deterministic() {
        let s = "some-string".to_string();
        assert_eq!(s.sha256_hash(), s.sha256_hash());
    }

    #[test]
    fn test_string_sha256_hash_differs_for_different_inputs() {
        let h1 = "input-one".to_string().sha256_hash();
        let h2 = "input-two".to_string().sha256_hash();
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_unvalidated_refresh_token_accepts_exactly_64_hex_chars() {
        let token_str = "a".repeat(64);
        let result = UnvalidatedRefreshToken::from_str(&token_str);
        assert!(result.is_ok());
        assert_eq!(&*result.unwrap(), token_str.as_str());
    }

    #[test]
    fn test_unvalidated_refresh_token_rejects_too_short() {
        let result = UnvalidatedRefreshToken::from_str("short");
        assert!(matches!(result, Err(InvalidRefreshToken::Length)));
    }

    #[test]
    fn test_unvalidated_refresh_token_rejects_too_long() {
        let result = UnvalidatedRefreshToken::from_str(&"a".repeat(65));
        assert!(matches!(result, Err(InvalidRefreshToken::Length)));
    }

    #[test]
    fn test_unvalidated_refresh_token_rejects_non_hex_characters() {
        let invalid = format!("{}z", "a".repeat(63));
        let result = UnvalidatedRefreshToken::from_str(&invalid);
        assert!(matches!(result, Err(InvalidRefreshToken::InvalidCharacters)));
    }

    #[test]
    fn test_unvalidated_refresh_token_trims_whitespace_before_validation() {
        let padded = format!(" {} ", "a".repeat(64));
        let result = UnvalidatedRefreshToken::from_str(&padded);
        assert!(result.is_ok());
    }
}
