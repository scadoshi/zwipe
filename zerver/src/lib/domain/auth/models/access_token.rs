use crate::domain::user::models::Username;
use email_address::EmailAddress;
use serde::{Deserialize, Serialize};
use std::{fmt::Display, str::FromStr};
use thiserror::Error;
use uuid::Uuid;

#[cfg(feature = "zerver")]
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};

// ========
//  errors
// ========

#[cfg(feature = "zerver")]
#[derive(Debug, Clone, Error)]
pub enum JwtSecretError {
    #[error("secret length must be 32+")]
    TooShort,
    #[error("secret must be present")]
    MissingSecret,
}

#[derive(Debug, Clone, Error)]
pub enum InvalidJwt {
    #[error("token must be present")]
    MissingToken,
    #[error("invalid token format")]
    Format,
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

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct UserClaims {
    pub user_id: Uuid,
    pub username: Username,
    pub email: EmailAddress,
    pub exp: i64,
    pub iat: i64,
}

// ======
//  main
// ======

/// jwt with format validation
/// (header.payload.signature)
#[derive(Debug, Clone, PartialEq)]
pub struct Jwt(String);

impl Jwt {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl FromStr for Jwt {
    type Err = InvalidJwt;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err(InvalidJwt::MissingToken);
        }
        if s.split(".").count() != 3 {
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

/// full access token
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AccessToken {
    pub jwt: Jwt,
    pub expires_at: i64,
}

#[cfg(feature = "zerver")]
impl AccessToken {
    pub fn generate(
        user_id: Uuid,
        username: Username,
        email: EmailAddress,
        secret: &JwtSecret,
    ) -> Result<AccessToken, InvalidJwt> {
        use chrono::Utc;

        let issued_at = Utc::now().timestamp() as i64;
        let expires_at = issued_at + 86400; // +24 hours

        let user_claims = UserClaims {
            user_id,
            username,
            email,
            exp: expires_at,
            iat: issued_at,
        };

        let jwt = Jwt::from_str(
            &encode(
                &Header::default(),
                &user_claims,
                &EncodingKey::from_secret(secret.as_ref()),
            )
            .map_err(|e| InvalidJwt::EncodingError(e))?,
        )?;

        Ok(AccessToken { jwt, expires_at })
    }

    pub fn validate(&self, secret: &JwtSecret) -> Result<UserClaims, jsonwebtoken::errors::Error> {
        let token_data = decode::<UserClaims>(
            &self.jwt.to_string(),
            &DecodingKey::from_secret(secret.as_ref()),
            &Validation::default(),
        )?;
        Ok(token_data.claims)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

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
        let user_id = Uuid::new_v4();
        let username = Username::new("testuser").unwrap();
        let email = EmailAddress::from_str("test@email.com").unwrap();
        let secret = JwtSecret::new("test-secret-that-is-long-enough-for-validation").unwrap();

        let result = AccessToken::generate(user_id, username, email, &secret);
        assert!(result.is_ok());
        let access_token = result.unwrap();
        assert!(!access_token.jwt.to_string().is_empty());
        assert_eq!(access_token.jwt.to_string().split('.').count(), 3); // JWT has 3 parts
    }

    #[test]
    fn test_generate_access_token_produces_consistent_results() {
        let user_id = Uuid::new_v4();
        let username = Username::new("testuser").unwrap();
        let email = EmailAddress::from_str("test@email.com").unwrap();
        let secret = JwtSecret::new("test-secret-that-is-long-enough-for-validation").unwrap();

        let token1 =
            AccessToken::generate(user_id.clone(), username.clone(), email.clone(), &secret)
                .unwrap();
        let token2 = AccessToken::generate(user_id, username, email, &secret).unwrap();
        assert_eq!(token1, token2);
    }

    #[test]
    fn test_generate_access_token_produces_unique_tokens_for_different_users() {
        let user_id1 = Uuid::new_v4();
        let user_id2 = Uuid::new_v4();
        let username1 = Username::new("user1").unwrap();
        let username2 = Username::new("user2").unwrap();
        let email1 = EmailAddress::from_str("user1@email.com").unwrap();
        let email2 = EmailAddress::from_str("user2@email.com").unwrap();
        let secret = JwtSecret::new("test-secret-that-is-long-enough-for-validation").unwrap();

        let token1 = AccessToken::generate(user_id1, username1, email1, &secret).unwrap();
        let token2 = AccessToken::generate(user_id2, username2, email2, &secret).unwrap();
        assert_ne!(token1, token2);
    }

    #[test]
    fn test_generate_access_token_produces_unique_tokens_for_different_secrets() {
        let user_id = Uuid::new_v4();
        let username = Username::new("testuser").unwrap();
        let email = EmailAddress::from_str("test@email.com").unwrap();
        let secret1 = JwtSecret::new("secret-1-that-is-long-enough-for-validation").unwrap();
        let secret2 = JwtSecret::new("secret-2-that-is-long-enough-for-validation").unwrap();

        let token1 =
            AccessToken::generate(user_id.clone(), username.clone(), email.clone(), &secret1)
                .unwrap();
        let token2 = AccessToken::generate(user_id, username, email, &secret2).unwrap();
        assert_ne!(token1, token2);
    }

    #[test]
    fn test_generate_access_token_normalizes_email_input() {
        let user_id = Uuid::new_v4();
        let username = Username::new("testuser").unwrap();
        let email = EmailAddress::from_str("TesT@eMaiL.Com").unwrap();
        let secret = JwtSecret::new("test-secret-that-is-long-enough-for-validation").unwrap();

        let token = AccessToken::generate(user_id, username, email, &secret).unwrap();
        let claims = token.validate(&secret).unwrap();
        assert_eq!(claims.email.to_string(), "test@email.com");
    }

    // =============================
    //  `AccessToken` validation tests
    // =============================

    #[test]
    fn test_validate_access_token_success_returns_correct_claims() {
        let user_id = Uuid::new_v4();
        let username = Username::new("testuser").unwrap();
        let email = EmailAddress::from_str("user@example.com").unwrap();
        let secret = JwtSecret::new("test-secret-that-is-long-enough-for-validation").unwrap();

        let token =
            AccessToken::generate(user_id.clone(), username.clone(), email.clone(), &secret)
                .unwrap();
        let claims = token.validate(&secret).unwrap();

        assert_eq!(claims.user_id, user_id);
        assert_eq!(claims.username, username);
        assert_eq!(claims.email, email);
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
        let user_id = Uuid::new_v4();
        let username = Username::new("testuser").unwrap();
        let email = EmailAddress::from_str("test@email.com").unwrap();
        let correct_secret =
            JwtSecret::new("correct-secret-that-is-long-enough-for-validation").unwrap();
        let wrong_secret =
            JwtSecret::new("wrong-secret-that-is-long-enough-for-validation").unwrap();

        let token = AccessToken::generate(user_id, username, email, &correct_secret).unwrap();
        let result = token.validate(&wrong_secret);
        assert!(result.is_err());
    }

    // ==========================
    //  `AccessToken` claims tests
    // ==========================

    #[test]
    fn test_access_token_claims_have_correct_expiration_and_issued_at() {
        let user_id = Uuid::new_v4();
        let username = Username::new("testuser").unwrap();
        let email = EmailAddress::from_str("test@email.com").unwrap();
        let secret = JwtSecret::new("test-secret-that-is-long-enough-for-validation").unwrap();

        let token = AccessToken::generate(user_id, username, email, &secret).unwrap();
        let claims = token.validate(&secret).unwrap();

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
        let user_id = Uuid::new_v4();
        let username = Username::new("testuser").unwrap();
        let email = EmailAddress::from_str("specific@example.com").unwrap();
        let secret = JwtSecret::new("test-secret-that-is-long-enough-for-validation").unwrap();

        let token =
            AccessToken::generate(user_id.clone(), username.clone(), email.clone(), &secret)
                .unwrap();
        let claims = token.validate(&secret).unwrap();

        assert_eq!(claims.user_id, user_id);
        assert_eq!(claims.username, username);
        assert_eq!(claims.email, email);
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
            let token =
                AccessToken::generate(user_id.clone(), username.clone(), email.clone(), &secret)
                    .unwrap();
            let claims = token.validate(&secret).unwrap();
            assert_eq!(claims.user_id, user_id);
        }
    }

    #[test]
    fn test_complete_authentication_flow_round_trip() {
        // Simulate a complete auth flow: generate token, validate it, extract claims
        let original_user_id = Uuid::new_v4();
        let original_username = Username::new("authuser").unwrap();
        let original_email = EmailAddress::from_str("auth@example.com").unwrap();
        let secret = JwtSecret::new("production-grade-secret-that-is-long-enough").unwrap();

        // Generate token (like during login)
        let token = AccessToken::generate(
            original_user_id.clone(),
            original_username.clone(),
            original_email.clone(),
            &secret,
        )
        .unwrap();

        // Validate token (like during protected route access)
        let claims = token.validate(&secret).unwrap();

        // Verify all data survived the round trip
        assert_eq!(claims.user_id, original_user_id);
        assert_eq!(claims.username, original_username);
        assert_eq!(claims.email, original_email);

        let now = chrono::Utc::now().timestamp();
        assert!(claims.exp > now);
        assert!(claims.iat <= now);
    }
}
