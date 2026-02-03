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
//!
//! # Token Rotation Flow
//!
//! 1. Client uses refresh token to request new access token
//! 2. Server validates refresh token (exists, not expired, hash matches)
//! 3. Server generates NEW refresh token and NEW access token
//! 4. Old refresh token is deleted from database
//! 5. Client receives both new tokens
//!
//! This prevents replay attacks - stolen refresh tokens become useless after one use.
//!
//! # Example
//!
//! ```rust,ignore
//! use zwipe::domain::auth::models::refresh_token::{RefreshToken, Sha256Hash};
//!
//! // Generate new token
//! let token = RefreshToken::generate();
//! let hash = token.sha256_hash();
//!
//! // Store hash in database, send value to client
//! ```

use std::str::FromStr;

use chrono::{Duration, NaiveDateTime, Utc};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

/// Refresh token lifespan (14 days).
///
/// Longer than access tokens (24h) to enable persistent sessions,
/// but short enough to limit exposure if compromised.
const REFRESH_TOKEN_LIFESPAN: Duration = Duration::days(14);

// =======
//  error
// =======

/// Errors that can occur while validating a refresh token string.
#[derive(Debug, Error)]
pub enum InvalidRefreshToken {
    /// Token string is not exactly 32 characters (hex-encoded 32 bytes).
    #[error("must be 32 characters")]
    Length,
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
///
/// # Security
///
/// The plaintext value is only sent to the client once during generation.
/// The database only stores the SHA-256 hash, preventing token theft from
/// database breaches.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct RefreshToken {
    /// The token value (64 hex characters = 32 random bytes).
    pub value: String,
    /// When this token expires (14 days from creation).
    pub expires_at: NaiveDateTime,
}

impl RefreshToken {
    /// Generates a new cryptographically-secure refresh token.
    ///
    /// Uses the system's secure random number generator to create
    /// 32 random bytes, hex-encodes them, and sets expiration to
    /// 14 days from now.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let token = RefreshToken::generate();
    /// assert_eq!(token.value.len(), 64); // 32 bytes * 2 hex chars
    /// ```
    pub fn generate() -> Self {
        let mut bytes = [0u8; 32];
        rand::rng().fill_bytes(&mut bytes);
        let value = hex::encode(bytes);
        let expires_at = Utc::now().naive_utc() + REFRESH_TOKEN_LIFESPAN;
        Self { value, expires_at }
    }
}

/// Trait for computing SHA-256 hashes of refresh tokens.
///
/// Used to hash tokens before database storage. The database never stores
/// plaintext refresh token values - only their SHA-256 hashes.
///
/// # Security Rationale
///
/// If an attacker gains read access to the database, they cannot use the
/// hashes to authenticate (hashes are one-way). The plaintext token values
/// only exist in client storage.
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
///
/// # Validation
///
/// Only checks length (must be exactly 32 characters). Does not verify:
/// - Whether the token exists in the database
/// - Whether the token is expired
/// - Whether the token hash matches
///
/// These checks happen at the service layer.
pub struct UnvalidatedRefreshToken(String);

impl UnvalidatedRefreshToken {
    /// Returns the token value as a string slice.
    pub fn value(&self) -> &str {
        &self.0
    }
}

impl FromStr for UnvalidatedRefreshToken {
    type Err = InvalidRefreshToken;

    /// Parses and validates a refresh token string.
    ///
    /// # Validation
    ///
    /// - Must be exactly 32 characters after trimming
    ///
    /// # Errors
    ///
    /// Returns [`InvalidRefreshToken::Length`] if the trimmed string is not 32 characters.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.trim().len() != 32 {
            return Err(InvalidRefreshToken::Length);
        }
        Ok(Self(s.to_string()))
    }
}
