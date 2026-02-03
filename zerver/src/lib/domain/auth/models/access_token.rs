//! JWT access token generation and validation.
//!
//! This module provides secure JWT (JSON Web Token) access tokens for authenticating
//! API requests. Access tokens are short-lived (24 hours) and contain user claims.
//!
//! # Token Structure
//!
//! Access tokens are JWTs with:
//! - **Header**: Algorithm (HS256) and token type
//! - **Claims**: User ID, username, email, issued at (iat), expiry (exp)
//! - **Signature**: HMAC-SHA256 signature using server secret
//!
//! # Security Features
//!
//! - **Short-Lived**: 24-hour expiry limits exposure if tokens are compromised
//! - **Signed**: HMAC-SHA256 signature prevents tampering
//! - **Stateless**: No database lookup needed for verification
//! - **Contains User Data**: Reduces database queries during request handling
//!
//! # Token Lifecycle
//!
//! 1. **Creation**: Generated during login/registration with user claims
//! 2. **Usage**: Sent in `Authorization: Bearer <token>` header
//! 3. **Validation**: Signature verified, expiry checked on each request
//! 4. **Expiry**: After 24h, client must use refresh token for new access token
//!
//! # JWT Secret
//!
//! The JWT secret is a server-side secret key used to sign tokens. It must be:
//! - At least 32 characters long
//! - Stored securely (environment variable, secrets manager)
//! - Never exposed to clients
//! - Rotated periodically for security
//!
//! # Example
//!
//! ```rust,ignore
//! use zwipe::domain::auth::models::access_token::{AccessToken, JwtSecret};
//!
//! // Generate token for user
//! let secret = JwtSecret::new(&std::env::var("JWT_SECRET")?)?;
//! let token = AccessToken::generate(&user, &secret)?;
//!
//! // Token is sent to client
//! println!("Access token: {}", token);
//!
//! // Later: validate token from request
//! let claims = token.validate(&secret)?;
//! println!("Authenticated user: {}", claims.username);
//! ```

use crate::domain::user::models::username::Username;
#[cfg(feature = "zerver")]
use crate::domain::user::models::User;
use chrono::{NaiveDateTime, Utc};
use email_address::EmailAddress;
#[cfg(feature = "zerver")]
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::{fmt::Display, str::FromStr};
use thiserror::Error;
use uuid::Uuid;

// ========
//  errors
// ========

#[cfg(feature = "zerver")]
/// Errors when constructing a JWT secret.
///
/// The JWT secret must meet minimum security requirements to ensure
/// token signatures cannot be easily brute-forced.
#[derive(Debug, Clone, Error)]
pub enum JwtSecretError {
    /// Secret is shorter than the minimum 32 characters.
    ///
    /// Secrets should be at least 32 characters (256 bits) to provide
    /// adequate security against brute-force attacks.
    #[error("secret length must be 32+")]
    TooShort,

    /// No secret was provided (empty string).
    #[error("secret must be present")]
    MissingSecret,
}

/// Errors when creating or validating JWT tokens.
///
/// These errors cover both token creation (encoding) and validation (decoding).
#[derive(Debug, Clone, Error)]
pub enum InvalidJwt {
    /// No token was provided (empty string).
    #[error("token must be present")]
    MissingToken,

    /// Token doesn't have the required JWT format (header.payload.signature).
    ///
    /// JWTs must have exactly 3 base64-encoded parts separated by dots.
    #[error("invalid token format")]
    Format,

    /// JWT encoding or decoding operation failed.
    ///
    /// This can indicate:
    /// - Invalid signature (token was tampered with)
    /// - Token expired
    /// - Invalid algorithm
    /// - Malformed base64 encoding
    #[cfg(feature = "zerver")]
    #[error(transparent)]
    EncodingError(jsonwebtoken::errors::Error),
}

#[cfg(feature = "zerver")]
impl From<jsonwebtoken::errors::Error> for InvalidJwt {
    fn from(value: jsonwebtoken::errors::Error) -> Self {
        Self::EncodingError(value)
    }
}

// ==========
//  newtypes
// ==========

#[cfg(feature = "zerver")]
/// Server-side secret key for signing and validating JWT tokens.
///
/// This secret is used to create HMAC-SHA256 signatures on JWTs, preventing
/// tampering. It must be kept secure and never exposed to clients.
///
/// # Security Requirements
///
/// - Minimum 32 characters (256 bits)
/// - Stored securely (environment variable, secrets manager)
/// - Rotated periodically
/// - Never logged or exposed in errors
///
/// # Example
///
/// ```rust,ignore
/// use zwipe::domain::auth::models::access_token::JwtSecret;
///
/// // Load from environment
/// let secret = JwtSecret::new(&std::env::var("JWT_SECRET")?)?;
///
/// // Use for signing and validation
/// let token = AccessToken::generate(&user, &secret)?;
/// let claims = token.value.validate(&secret)?;
/// ```
#[derive(Debug, Clone)]
pub struct JwtSecret(String);

#[cfg(feature = "zerver")]
impl AsRef<[u8]> for JwtSecret {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

#[cfg(feature = "zerver")]
impl JwtSecret {
    /// Creates a new JWT secret with validation.
    ///
    /// # Arguments
    ///
    /// * `raw` - The secret string (minimum 32 characters)
    ///
    /// # Errors
    ///
    /// Returns [`JwtSecretError`] if the secret is empty or shorter than 32 characters.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let secret = JwtSecret::new("my-super-secret-key-that-is-at-least-32-chars-long")?;
    /// ```
    pub fn new(raw: &str) -> Result<Self, JwtSecretError> {
        if raw.is_empty() {
            return Err(JwtSecretError::MissingSecret);
        }
        if raw.len() < 32 {
            return Err(JwtSecretError::TooShort);
        }
        Ok(Self(raw.to_string()))
    }
}

/// JWT claims containing user information.
///
/// These claims are embedded in the JWT payload and used to identify the
/// authenticated user without database lookups on every request.
///
/// # Standard Claims
///
/// - `iat` (issued at): Unix timestamp when token was created
/// - `exp` (expiry): Unix timestamp when token expires (24h after iat)
///
/// # Custom Claims
///
/// - `user_id`: User's unique identifier
/// - `username`: User's username (cached for display)
/// - `email`: User's email (cached for display)
///
/// # Security Note
///
/// Claims are Base64-encoded, NOT encrypted. Don't include sensitive data
/// like passwords or private information that shouldn't be readable by clients.
///
/// # Example
///
/// ```rust,ignore
/// // Claims are extracted from validated tokens
/// let claims: UserClaims = token.value.validate(&secret)?;
///
/// println!("Authenticated user: {} ({})", claims.username, claims.user_id);
/// println!("Token expires: {}", chrono::NaiveDateTime::from_timestamp(claims.exp, 0));
/// ```
#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct UserClaims {
    /// Unique identifier for the authenticated user.
    pub user_id: Uuid,

    /// Username of the authenticated user (cached from database).
    pub username: Username,

    /// Email of the authenticated user (cached from database).
    pub email: EmailAddress,

    /// Expiry time as Unix timestamp (seconds since epoch).
    ///
    /// Tokens expire 24 hours after issuance.
    pub exp: i64,

    /// Issued at time as Unix timestamp (seconds since epoch).
    pub iat: i64,
}

// ======
//  main
// ======

/// A validated JWT token string with the correct format.
///
/// This value object wraps a JWT string and guarantees it has the proper
/// structure: `header.payload.signature` (exactly 3 base64-encoded parts
/// separated by dots).
///
/// # Format
///
/// ```text
/// eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJ1c2VyX2lkIjoiLi4uIn0.SIGNATURE
/// └─────────── header ────────────┘ └──── payload ────┘ └─ signature ─┘
/// ```
///
/// # Validation
///
/// Format validation happens on construction (via `FromStr`), but signature
/// and expiry validation requires calling [`validate()`](Self::validate) with
/// the JWT secret.
///
/// # Example
///
/// ```rust,ignore
/// use zwipe::domain::auth::models::access_token::Jwt;
/// use std::str::FromStr;
///
/// // Parse JWT from string
/// let jwt = Jwt::from_str(token_string)?;
///
/// // Validate signature and extract claims
/// let claims = jwt.validate(&secret)?;
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Jwt(String);

impl Jwt {
    /// Returns the JWT token as a string slice.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let token_str: &str = jwt.as_str();
    /// // Send in Authorization header: "Bearer {token_str}"
    /// ```
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Validates the JWT signature and extracts user claims.
    ///
    /// This performs:
    /// 1. Signature verification using HMAC-SHA256
    /// 2. Expiry time check
    /// 3. Claims deserialization
    ///
    /// # Arguments
    ///
    /// * `secret` - The JWT secret used to sign the token
    ///
    /// # Returns
    ///
    /// Returns the [`UserClaims`] if validation succeeds.
    ///
    /// # Errors
    ///
    /// Returns [`jsonwebtoken::errors::Error`] if:
    /// - Signature is invalid (token was tampered with)
    /// - Token has expired
    /// - Token uses wrong algorithm
    /// - Claims cannot be deserialized
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let claims = jwt.validate(&secret)?;
    /// println!("Authenticated as: {}", claims.username);
    /// ```
    #[cfg(feature = "zerver")]
    pub fn validate(&self, secret: &JwtSecret) -> Result<UserClaims, jsonwebtoken::errors::Error> {
        let token_data = decode::<UserClaims>(
            &self.to_string(),
            &DecodingKey::from_secret(secret.as_ref()),
            &Validation::default(),
        )?;
        Ok(token_data.claims)
    }
}

impl FromStr for Jwt {
    type Err = InvalidJwt;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err(InvalidJwt::MissingToken);
        }
        if s.split('.').count() != 3 {
            return Err(InvalidJwt::Format);
        }
        Ok(Self(s.to_string()))
    }
}

impl Display for Jwt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Serialize for Jwt {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for Jwt {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Jwt::from_str(s.as_str()).map_err(serde::de::Error::custom)
    }
}

/// A complete access token with JWT value and expiry time.
///
/// This is the primary authentication credential returned to clients after login
/// or registration. It contains both the JWT token string and metadata about when
/// it expires.
///
/// # Token Lifecycle
///
/// 1. **Generation**: Created during authentication with 24-hour expiry
/// 2. **Client Storage**: Client stores token securely (localStorage, secure cookie)
/// 3. **Request Authentication**: Client sends in `Authorization: Bearer <token>` header
/// 4. **Server Validation**: Server validates signature and expiry on each request
/// 5. **Expiry**: After 24h, client must use refresh token to get new access token
///
/// # Usage in API Requests
///
/// ```text
/// GET /api/decks HTTP/1.1
/// Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
/// ```
///
/// # Security
///
/// - **Short-Lived**: 24-hour expiry limits damage if token is compromised
/// - **Signed**: Cannot be forged without the server secret
/// - **Stateless**: No database lookup needed for validation
///
/// # Example
///
/// ```rust,ignore
/// use zwipe::domain::auth::models::access_token::AccessToken;
///
/// // Generate token for user
/// let token = AccessToken::generate(&user, &jwt_secret)?;
///
/// // Send to client
/// println!("Token: {}", token.value);
/// println!("Expires: {}", token.expires_at);
///
/// // Client uses token in requests
/// // Later: validate incoming token
/// let claims = token.value.validate(&jwt_secret)?;
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AccessToken {
    /// The JWT token string.
    ///
    /// This is sent to the client and included in API request headers.
    pub value: Jwt,

    /// When the token expires (24 hours from issuance).
    ///
    /// After this time, the token cannot be used and the client must
    /// use the refresh token to obtain a new access token.
    pub expires_at: NaiveDateTime,
}

impl AccessToken {
    /// Creates a new access token from components.
    ///
    /// Typically used when reconstructing from database or for testing.
    /// Most production code should use [`generate()`](Self::generate) instead.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let token = AccessToken::new(jwt, expires_at);
    /// ```
    #[cfg(feature = "zerver")]
    pub fn new(value: Jwt, expires_at: NaiveDateTime) -> Self {
        Self { value, expires_at }
    }

    /// Generates a new access token for a user.
    ///
    /// Creates a JWT with user claims (id, username, email) and a 24-hour expiry time.
    /// The token is signed with HMAC-SHA256 using the provided secret.
    ///
    /// # Arguments
    ///
    /// * `user` - The user for whom to generate the token
    /// * `secret` - The JWT secret for signing
    ///
    /// # Returns
    ///
    /// Returns an [`AccessToken`] with the JWT and expiry time.
    ///
    /// # Errors
    ///
    /// Returns [`InvalidJwt`] if JWT encoding fails (extremely rare).
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use zwipe::domain::auth::models::access_token::AccessToken;
    ///
    /// let token = AccessToken::generate(&user, &jwt_secret)?;
    /// // Token expires 24 hours from now
    /// ```
    #[cfg(feature = "zerver")]
    pub fn generate(user: &User, secret: &JwtSecret) -> Result<AccessToken, InvalidJwt> {
        use chrono::{Duration, Utc};

        let issued_at = Utc::now().naive_utc();
        let expires_at = issued_at + Duration::hours(24);

        let user_claims = UserClaims {
            user_id: user.id,
            username: user.username.clone(),
            email: user.email.clone(),
            exp: expires_at.and_utc().timestamp(),
            iat: issued_at.and_utc().timestamp(),
        };

        let value = Jwt::from_str(
            &encode(
                &Header::default(),
                &user_claims,
                &EncodingKey::from_secret(secret.as_ref()),
            )
            .map_err(InvalidJwt::EncodingError)?,
        )?;

        Ok(AccessToken::new(value, expires_at))
    }

    /// Checks if the access token has expired (current time >= expires_at).
    pub fn is_expired(&self) -> bool {
        self.expires_at < Utc::now().naive_utc()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    // ========================
    //  `JwtSecret` test
    // ========================

    #[test]
    fn test_jwt_secret_new_accepts_valid_secret() {
        let secret = JwtSecret::new("this-is-a-valid-secret-that-is-long-enough");
        assert!(secret.is_ok());
    }

    #[test]
    fn test_jwt_secret_new_rejects_empty_secret() {
        let secret = JwtSecret::new("");
        assert!(secret.is_err());
        assert!(matches!(secret.unwrap_err(), JwtSecretError::MissingSecret));
    }

    #[test]
    fn test_jwt_secret_new_rejects_short_secret() {
        let secret = JwtSecret::new("short");
        assert!(secret.is_err());
        assert!(matches!(secret.unwrap_err(), JwtSecretError::TooShort));
    }

    #[test]
    fn test_jwt_secret_new_rejects_secret_exactly_31_chars() {
        let secret = JwtSecret::new("1234567890123456789012345678901"); // 31 chars
        assert!(secret.is_err());
        assert!(matches!(secret.unwrap_err(), JwtSecretError::TooShort));
    }

    #[test]
    fn test_jwt_secret_new_accepts_secret_exactly_32_chars() {
        let secret = JwtSecret::new("12345678901234567890123456789012"); // 32 chars
        assert!(secret.is_ok());
    }

    #[test]
    fn test_jwt_secret_as_ref_returns_bytes() {
        let secret = JwtSecret::new("test-secret-that-is-long-enough-for-validation").unwrap();
        let bytes = secret.as_ref();
        assert_eq!(bytes, b"test-secret-that-is-long-enough-for-validation");
    }

    // ================
    //  `Jwt` tests
    // ================

    #[test]
    fn test_access_token_new_accepts_valid_token() {
        let token = Jwt::from_str("header.payload.signature");
        assert!(token.is_ok());
    }

    #[test]
    fn test_access_token_new_rejects_empty_token() {
        let token = Jwt::from_str("");
        assert!(token.is_err());
        assert!(matches!(token.unwrap_err(), InvalidJwt::MissingToken));
    }

    #[test]
    fn test_access_token_new_rejects_token_with_too_few_parts() {
        let token = Jwt::from_str("header.payload");
        assert!(token.is_err());
        assert!(matches!(token.unwrap_err(), InvalidJwt::Format));
    }

    #[test]
    fn test_access_token_new_rejects_token_with_too_many_parts() {
        let token = Jwt::from_str("header.payload.signature.extra");
        assert!(token.is_err());
        assert!(matches!(token.unwrap_err(), InvalidJwt::Format));
    }

    #[test]
    fn test_access_token_display_formats_correctly() {
        let token_str = "header.payload.signature";
        let token = Jwt::from_str(token_str).unwrap();
        assert_eq!(token.to_string(), token_str);
    }

    #[test]
    fn test_access_token_partial_eq_works() {
        let token1 = Jwt::from_str("header.payload.signature").unwrap();
        let token2 = Jwt::from_str("header.payload.signature").unwrap();
        let token3 = Jwt::from_str("different.payload.signature").unwrap();

        assert_eq!(token1, token2);
        assert_ne!(token1, token3);
    }

    // ====================
    //  `UserClaims` tests
    // ====================

    #[test]
    fn test_user_claims_serialization_round_trip() {
        let user_id = uuid::Uuid::new_v4();
        let username = Username::new("test").unwrap();
        let email = EmailAddress::from_str("test@example.com").unwrap();
        let claims = UserClaims {
            user_id,
            username,
            email,
            exp: 1234567890,
            iat: 1234567890,
        };

        let serialized = serde_json::to_string(&claims).unwrap();
        let deserialized: UserClaims = serde_json::from_str(&serialized).unwrap();

        assert_eq!(claims, deserialized);
    }

    #[test]
    fn test_user_claims_partial_eq_works() {
        let id1 = uuid::Uuid::new_v4();
        let id2 = uuid::Uuid::new_v4();
        let username = Username::new("test").unwrap();
        let email = EmailAddress::from_str("test@example.com").unwrap();
        let claims1 = UserClaims {
            user_id: id1.clone(),
            username: username.clone(),
            email: email.clone(),
            exp: 1234567890,
            iat: 1234567890,
        };
        let claims2 = UserClaims {
            user_id: id1,
            username: username.clone(),
            email: email.clone(),
            exp: 1234567890,
            iat: 1234567890,
        };
        let claims3 = UserClaims {
            user_id: id2,
            username: username.clone(),
            email: email.clone(),
            exp: 1234567890,
            iat: 1234567890,
        };

        assert_eq!(claims1, claims2);
        assert_ne!(claims1, claims3);
    }

    // =============================
    //  `AccessToken` generation tests
    // =============================

    #[test]
    fn test_generate_access_token_success_creates_valid_tokens() {
        let user = User::new(
            Uuid::new_v4(),
            Username::new("testuser").unwrap(),
            EmailAddress::from_str("test@email.com").unwrap(),
        );

        let secret = JwtSecret::new("test-secret-that-is-long-enough-for-validation").unwrap();

        let result = AccessToken::generate(&user, &secret);
        assert!(result.is_ok());
        let access_token = result.unwrap();
        assert!(!access_token.value.to_string().is_empty());
        assert_eq!(access_token.value.to_string().split('.').count(), 3); // JWT has 3 parts
    }

    #[test]
    fn test_generate_access_token_produces_consistent_results() {
        let user = User::new(
            Uuid::new_v4(),
            Username::new("testuser").unwrap(),
            EmailAddress::from_str("test@email.com").unwrap(),
        );

        let secret = JwtSecret::new("test-secret-that-is-long-enough-for-validation").unwrap();

        let token1 = AccessToken::generate(&user, &secret).unwrap();
        let token2 = AccessToken::generate(&user, &secret).unwrap();
        assert_eq!(token1, token2);
    }

    #[test]
    fn test_generate_access_token_produces_unique_tokens_for_different_users() {
        let user1 = User::new(
            Uuid::new_v4(),
            Username::new("testuser1").unwrap(),
            EmailAddress::from_str("test1@email.com").unwrap(),
        );

        let user2 = User::new(
            Uuid::new_v4(),
            Username::new("testuser2").unwrap(),
            EmailAddress::from_str("test2@email.com").unwrap(),
        );

        let secret = JwtSecret::new("test-secret-that-is-long-enough-for-validation").unwrap();

        let token1 = AccessToken::generate(&user1, &secret).unwrap();
        let token2 = AccessToken::generate(&user2, &secret).unwrap();
        assert_ne!(token1, token2);
    }

    #[test]
    fn test_generate_access_token_produces_unique_tokens_for_different_secrets() {
        let user = User::new(
            Uuid::new_v4(),
            Username::new("testuser").unwrap(),
            EmailAddress::from_str("test@email.com").unwrap(),
        );
        let secret1 = JwtSecret::new("secret-1-that-is-long-enough-for-validation").unwrap();
        let secret2 = JwtSecret::new("secret-2-that-is-long-enough-for-validation").unwrap();

        let token1 = AccessToken::generate(&user, &secret1).unwrap();
        let token2 = AccessToken::generate(&user, &secret2).unwrap();
        assert_ne!(token1, token2);
    }

    #[test]
    fn test_generate_access_token_normalizes_email_input() {
        let user = User::new(
            Uuid::new_v4(),
            Username::new("testuser").unwrap(),
            EmailAddress::from_str("test@email.com").unwrap(),
        );
        let secret = JwtSecret::new("test-secret-that-is-long-enough-for-validation").unwrap();

        let token = AccessToken::generate(&user, &secret).unwrap();
        let claims = token.value.validate(&secret).unwrap();
        assert_eq!(claims.email.to_string(), "test@email.com");
    }

    // =============================
    //  `AccessToken` validation tests
    // =============================

    #[test]
    fn test_validate_access_token_success_returns_correct_claims() {
        let user = User::new(
            Uuid::new_v4(),
            Username::new("testuser").unwrap(),
            EmailAddress::from_str("test@email.com").unwrap(),
        );
        let secret = JwtSecret::new("test-secret-that-is-long-enough-for-validation").unwrap();

        let token = AccessToken::generate(&user, &secret).unwrap();
        let claims = token.value.validate(&secret).unwrap();

        assert_eq!(claims.user_id, user.id);
        assert_eq!(claims.username, user.username);
        assert_eq!(claims.email, user.email);
    }

    #[test]
    fn test_validate_access_token_rejects_malformed_tokens() {
        // These tests verify that Jwt format validation works at construction time
        // Format validation happens in Jwt::from_str before AccessToken can be created

        // Too many sections - valid JWT should have exactly 3 parts
        assert!(Jwt::from_str("token.with.too.many.sections").is_err());

        // Too few sections
        assert!(Jwt::from_str("too.few").is_err());

        // Empty token
        assert!(Jwt::from_str("").is_err());
    }

    #[test]
    fn test_validate_access_token_rejects_wrong_secret() {
        let user = User::new(
            Uuid::new_v4(),
            Username::new("testuser").unwrap(),
            EmailAddress::from_str("test@email.com").unwrap(),
        );

        let correct_secret =
            JwtSecret::new("correct-secret-that-is-long-enough-for-validation").unwrap();
        let wrong_secret =
            JwtSecret::new("wrong-secret-that-is-long-enough-for-validation").unwrap();

        let token = AccessToken::generate(&user, &correct_secret).unwrap();
        let result = token.value.validate(&wrong_secret);
        assert!(result.is_err());
    }

    // ==========================
    //  `AccessToken` claims tests
    // ==========================

    #[test]
    fn test_access_token_claims_have_correct_expiration_and_issued_at() {
        let user = User::new(
            Uuid::new_v4(),
            Username::new("testuser").unwrap(),
            EmailAddress::from_str("test@email.com").unwrap(),
        );
        let secret = JwtSecret::new("test-secret-that-is-long-enough-for-validation").unwrap();

        let token = AccessToken::generate(&user, &secret).unwrap();
        let claims = token.value.validate(&secret).unwrap();

        let now = chrono::Utc::now().timestamp();

        // Token should expire in the future
        assert!(claims.exp > now);

        // Token should be issued now or in the past
        assert!(claims.iat <= now);

        // Token should have 24-hour duration
        assert_eq!(claims.exp - claims.iat, 86400);
    }

    #[test]
    fn test_access_token_claims_match_input_data() {
        let user = User::new(
            Uuid::new_v4(),
            Username::new("testuser").unwrap(),
            EmailAddress::from_str("test@email.com").unwrap(),
        );
        let secret = JwtSecret::new("test-secret-that-is-long-enough-for-validation").unwrap();

        let token = AccessToken::generate(&user, &secret).unwrap();
        let claims = token.value.validate(&secret).unwrap();

        assert_eq!(claims.user_id, user.id);
        assert_eq!(claims.username, user.username);
        assert_eq!(claims.email, user.email);
    }

    // =====================
    //  integration tests
    // =====================

    #[test]
    fn test_generate_and_validate_round_trip_with_multiple_user_ids() {
        let user_ids = vec![Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4()]; // Multiple random UUIDs

        let username = Username::new("testuser").unwrap();
        let email = EmailAddress::from_str("test@email.com").unwrap();

        let secret = JwtSecret::new("test-secret-that-is-long-enough-for-validation").unwrap();

        for user_id in user_ids {
            let user = User::new(user_id, username.clone(), email.clone());
            let token = AccessToken::generate(&user, &secret).unwrap();
            let claims = token.value.validate(&secret).unwrap();
            assert_eq!(claims.user_id, user_id);
        }
    }

    #[test]
    fn test_complete_authentication_flow_round_trip() {
        // Simulate a complete auth flow: generate token, validate it, extract claims
        let original_user = User::new(
            Uuid::new_v4(),
            Username::new("authuser").unwrap(),
            EmailAddress::from_str("auth@example.com").unwrap(),
        );
        let secret = JwtSecret::new("production-grade-secret-that-is-long-enough").unwrap();

        // Generate token (like during login)
        let token = AccessToken::generate(&original_user, &secret).unwrap();

        // Validate token (like during protected route access)
        let claims = token.value.validate(&secret).unwrap();

        // Verify all data survived the round trip
        assert_eq!(claims.user_id, original_user.id);
        assert_eq!(claims.username, original_user.username);
        assert_eq!(claims.email, original_user.email);

        let now = chrono::Utc::now().timestamp();
        assert!(claims.exp > now);
        assert!(claims.iat <= now);
    }
}
