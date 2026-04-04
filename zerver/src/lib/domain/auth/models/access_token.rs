//! JWT access token generation and validation.
//!
//! Re-exports pure types from zwipe-core and provides server-only operations:
//! - [`JwtSecret`]: Server-side signing key
//! - [`JwtValidate`]: Extension trait for JWT signature verification
//! - [`AccessTokenExt`]: Extension trait for token generation

#[cfg(feature = "zerver")]
use zwipe_core::domain::auth::models::access_token::{
    AccessToken, Jwt, UserClaims,
    InvalidJwt as CoreInvalidJwt,
};

#[cfg(feature = "zerver")]
use zwipe_core::domain::user::User;
#[cfg(feature = "zerver")]
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
#[cfg(feature = "zerver")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "zerver")]
use std::str::FromStr;
#[cfg(feature = "zerver")]
use thiserror::Error;

// ========
//  errors
// ========

#[cfg(feature = "zerver")]
/// Errors when constructing a JWT secret.
#[derive(Debug, Clone, Error)]
pub enum JwtSecretError {
    /// Secret is shorter than the minimum 32 characters.
    #[error("secret length must be 32+")]
    TooShort,

    /// No secret was provided (empty string).
    #[error("secret must be present")]
    MissingSecret,
}

#[cfg(feature = "zerver")]
/// Errors when creating or validating JWT tokens.
///
/// Extends the core [`CoreInvalidJwt`] with the server-only `EncodingError` variant.
#[derive(Debug, Clone, Error)]
pub enum InvalidJwt {
    /// No token was provided (empty string).
    #[error("token must be present")]
    MissingToken,

    /// Token doesn't have the required JWT format (header.payload.signature).
    #[error("invalid token format")]
    Format,

    /// JWT encoding or decoding operation failed.
    #[error(transparent)]
    EncodingError(jsonwebtoken::errors::Error),
}

#[cfg(feature = "zerver")]
impl From<CoreInvalidJwt> for InvalidJwt {
    fn from(e: CoreInvalidJwt) -> Self {
        match e {
            CoreInvalidJwt::MissingToken => Self::MissingToken,
            CoreInvalidJwt::Format => Self::Format,
        }
    }
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
    /// # Errors
    ///
    /// Returns [`JwtSecretError`] if the secret is empty or shorter than 32 characters.
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

// ===================
//  extension traits
// ===================

#[cfg(feature = "zerver")]
/// Extension trait for validating JWT signatures on the server.
pub trait JwtValidate {
    /// Validates the JWT signature and extracts user claims.
    fn validate(&self, secret: &JwtSecret) -> Result<UserClaims, jsonwebtoken::errors::Error>;
}

#[cfg(feature = "zerver")]
impl JwtValidate for Jwt {
    fn validate(&self, secret: &JwtSecret) -> Result<UserClaims, jsonwebtoken::errors::Error> {
        let token_data = decode::<UserClaims>(
            &self.to_string(),
            &DecodingKey::from_secret(secret.as_ref()),
            &Validation::default(),
        )?;
        Ok(token_data.claims)
    }
}

#[cfg(feature = "zerver")]
/// Extension trait for server-side access token operations.
pub trait AccessTokenExt {
    /// Generates a new access token for a user.
    fn generate(user: &User, secret: &JwtSecret) -> Result<AccessToken, InvalidJwt>;
}

#[cfg(feature = "zerver")]
impl AccessTokenExt for AccessToken {
    fn generate(
        user: &User,
        secret: &JwtSecret,
    ) -> Result<AccessToken, InvalidJwt> {
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

        Ok(AccessToken { value, expires_at })
    }
}

// =======================
//  serde for InvalidJwt
// =======================

#[cfg(feature = "zerver")]
impl Serialize for InvalidJwt {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

#[cfg(feature = "zerver")]
impl<'de> Deserialize<'de> for InvalidJwt {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "token must be present" => Ok(Self::MissingToken),
            "invalid token format" => Ok(Self::Format),
            _ => Err(serde::de::Error::custom("unknown InvalidJwt variant")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use email_address::EmailAddress;
    use std::str::FromStr;
    use uuid::Uuid;
    use zwipe_core::domain::user::username::Username;

    // ========================
    //  `JwtSecret` tests
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
        assert!(!access_token.value.is_empty());
        assert_eq!(access_token.value.split('.').count(), 3);
    }

    #[test]
    fn test_generate_access_token_produces_consistent_claims() {
        let user_id = Uuid::new_v4();
        let user = User::new(
            user_id,
            Username::new("testuser").unwrap(),
            EmailAddress::from_str("test@email.com").unwrap(),
        );

        let secret = JwtSecret::new("test-secret-that-is-long-enough-for-validation").unwrap();

        let token1 = AccessToken::generate(&user, &secret).unwrap();
        let token2 = AccessToken::generate(&user, &secret).unwrap();

        let claims1 = token1.value.validate(&secret).unwrap();
        let claims2 = token2.value.validate(&secret).unwrap();
        assert_eq!(claims1.user_id, claims2.user_id);
        assert_eq!(claims1.username, claims2.username);
        assert_eq!(claims1.email, claims2.email);
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
        assert!(claims.exp > now);
        assert!(claims.iat <= now);
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
        let user_ids = vec![Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4()];

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
        let original_user = User::new(
            Uuid::new_v4(),
            Username::new("authuser").unwrap(),
            EmailAddress::from_str("auth@example.com").unwrap(),
        );
        let secret = JwtSecret::new("production-grade-secret-that-is-long-enough").unwrap();

        let token = AccessToken::generate(&original_user, &secret).unwrap();
        let claims = token.value.validate(&secret).unwrap();

        assert_eq!(claims.user_id, original_user.id);
        assert_eq!(claims.username, original_user.username);
        assert_eq!(claims.email, original_user.email);

        let now = chrono::Utc::now().timestamp();
        assert!(claims.exp > now);
        assert!(claims.iat <= now);
    }
}
