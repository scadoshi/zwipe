use std::fmt::Display;

use email_address::EmailAddress;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

// =============================================================================
// ERROR TYPES
// =============================================================================

/// Errors to return while using new constructor for JwtSecret
#[derive(Debug, Clone, Error)]
pub enum JwtSecretError {
    #[error("Secret length must be 32+")]
    TooShort,
    #[error("Secret must be present")]
    MissingSecret,
}

/// Errors to return while using new constructor for Jwt
#[derive(Debug, Clone, Error)]
pub enum JwtError {
    #[error("Token must be present")]
    MissingToken,
    #[error("Invalid token format")]
    InvalidFormat,
    #[error(transparent)]
    EncodingError(jsonwebtoken::errors::Error),
}

// =============================================================================
// DOMAIN NEWTYPES
// =============================================================================
/// JWT secret with minimum 32-character cryptographic strength requirement
#[derive(Debug, Clone)]
pub struct JwtSecret(String);

impl AsRef<[u8]> for JwtSecret {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

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

/// User claims embedded in JWT tokens
#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct UserClaims {
    pub user_id: Uuid,
    pub email: EmailAddress,
    pub exp: usize, // expiration timestamp
    pub iat: usize, // issued at timestamp
}

// =============================================================================
// REQUEST/RESPONSE TYPES
// =============================================================================

/// JWT generation response containing token and expiration time
#[derive(Debug, Clone)]
pub struct JwtCreationResponse {
    pub jwt: Jwt,
    pub expires_at: usize,
}

// =============================================================================
// MAIN DOMAIN ENTITIES
// =============================================================================

/// JWT token with format validation (header.payload.signature)
#[derive(Debug, Clone, PartialEq)]
pub struct Jwt(String);

impl Display for Jwt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Jwt {
    /// Generates JWT token with 24-hour expiration
    pub fn generate(
        user_id: Uuid,
        email: EmailAddress,
        secret: JwtSecret,
    ) -> Result<JwtCreationResponse, JwtError> {
        let issued_at = chrono::Utc::now().timestamp() as usize;
        let expires_at = issued_at + 86400; // 24 hours

        let user_claims = UserClaims {
            user_id,
            email,
            exp: expires_at,
            iat: issued_at,
        };

        let jwt = Jwt::new(
            &jsonwebtoken::encode(
                &jsonwebtoken::Header::default(),
                &user_claims,
                &jsonwebtoken::EncodingKey::from_secret(secret.0.as_ref()),
            )
            .map_err(|e| JwtError::EncodingError(e))?,
        )?;

        Ok(JwtCreationResponse { jwt, expires_at })
    }

    pub fn new(raw: &str) -> Result<Self, JwtError> {
        if raw.is_empty() {
            return Err(JwtError::MissingToken);
        }
        if raw.split(".").count() != 3 {
            return Err(JwtError::InvalidFormat);
        }
        Ok(Self(raw.to_string()))
    }

    pub fn validate(&self, secret: &JwtSecret) -> Result<UserClaims, jsonwebtoken::errors::Error> {
        let token_data = jsonwebtoken::decode::<UserClaims>(
            &self.to_string(),
            &jsonwebtoken::DecodingKey::from_secret(secret.as_ref()),
            &jsonwebtoken::Validation::default(),
        )?;
        Ok(token_data.claims)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    // ================================
    // JwtSecret Tests
    // ================================

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

    // ================================
    // Jwt Tests
    // ================================

    #[test]
    fn test_jwt_new_accepts_valid_token() {
        let token = Jwt::new("header.payload.signature");
        assert!(token.is_ok());
    }

    #[test]
    fn test_jwt_new_rejects_empty_token() {
        let token = Jwt::new("");
        assert!(token.is_err());
        assert!(matches!(token.unwrap_err(), JwtError::MissingToken));
    }

    #[test]
    fn test_jwt_new_rejects_token_with_too_few_parts() {
        let token = Jwt::new("header.payload");
        assert!(token.is_err());
        assert!(matches!(token.unwrap_err(), JwtError::InvalidFormat));
    }

    #[test]
    fn test_jwt_new_rejects_token_with_too_many_parts() {
        let token = Jwt::new("header.payload.signature.extra");
        assert!(token.is_err());
        assert!(matches!(token.unwrap_err(), JwtError::InvalidFormat));
    }

    #[test]
    fn test_jwt_display_formats_correctly() {
        let token_str = "header.payload.signature";
        let token = Jwt::new(token_str).unwrap();
        assert_eq!(token.to_string(), token_str);
    }

    #[test]
    fn test_jwt_partial_eq_works() {
        let token1 = Jwt::new("header.payload.signature").unwrap();
        let token2 = Jwt::new("header.payload.signature").unwrap();
        let token3 = Jwt::new("different.payload.signature").unwrap();

        assert_eq!(token1, token2);
        assert_ne!(token1, token3);
    }

    // ================================
    // UserClaims Tests
    // ================================

    #[test]
    fn test_user_claims_serialization_round_trip() {
        let user_id = uuid::Uuid::new_v4();
        let email = EmailAddress::from_str("test@example.com").unwrap();
        let claims = UserClaims {
            user_id,
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
        let claims1 = UserClaims {
            user_id: id1.clone(),
            email: EmailAddress::from_str("test@example.com").unwrap(),
            exp: 1234567890,
            iat: 1234567890,
        };
        let claims2 = UserClaims {
            user_id: id1,
            email: EmailAddress::from_str("test@example.com").unwrap(),
            exp: 1234567890,
            iat: 1234567890,
        };
        let claims3 = UserClaims {
            user_id: id2,
            email: EmailAddress::from_str("test@example.com").unwrap(),
            exp: 1234567890,
            iat: 1234567890,
        };

        assert_eq!(claims1, claims2);
        assert_ne!(claims1, claims3);
    }

    // ================================
    // JWT Generation Tests
    // ================================

    #[test]
    fn test_generate_jwt_success_creates_valid_tokens() {
        let user_id = Uuid::new_v4();
        let email = EmailAddress::from_str("test@email.com").unwrap();
        let secret = JwtSecret::new("test-secret-that-is-long-enough-for-validation").unwrap();

        let result = Jwt::generate(user_id, email, secret);
        assert!(result.is_ok());
        let token = result.unwrap().jwt;
        assert!(!token.to_string().is_empty());
        assert_eq!(token.to_string().split('.').count(), 3); // JWT has 3 parts
    }

    #[test]
    fn test_generate_jwt_produces_consistent_results() {
        let user_id = Uuid::new_v4();
        let email = EmailAddress::from_str("test@email.com").unwrap();
        let secret = JwtSecret::new("test-secret-that-is-long-enough-for-validation").unwrap();

        let token1 = Jwt::generate(user_id.clone(), email.clone(), secret.clone())
            .unwrap()
            .jwt;
        let token2 = Jwt::generate(user_id, email, secret).unwrap().jwt;
        assert_eq!(token1, token2);
    }

    #[test]
    fn test_generate_jwt_produces_unique_tokens_for_different_users() {
        let user_id1 = Uuid::new_v4();
        let user_id2 = Uuid::new_v4();
        let email1 = EmailAddress::from_str("user1@email.com").unwrap();
        let email2 = EmailAddress::from_str("user2@email.com").unwrap();
        let secret = JwtSecret::new("test-secret-that-is-long-enough-for-validation").unwrap();

        let token1 = Jwt::generate(user_id1, email1, secret.clone()).unwrap().jwt;
        let token2 = Jwt::generate(user_id2, email2, secret).unwrap().jwt;
        assert_ne!(token1, token2);
    }

    #[test]
    fn test_generate_jwt_produces_unique_tokens_for_different_secrets() {
        let user_id = Uuid::new_v4();
        let email = EmailAddress::from_str("test@email.com").unwrap();
        let secret1 = JwtSecret::new("secret-1-that-is-long-enough-for-validation").unwrap();
        let secret2 = JwtSecret::new("secret-2-that-is-long-enough-for-validation").unwrap();

        let token1 = Jwt::generate(user_id.clone(), email.clone(), secret1)
            .unwrap()
            .jwt;
        let token2 = Jwt::generate(user_id, email, secret2).unwrap().jwt;
        assert_ne!(token1, token2);
    }

    #[test]
    fn test_generate_jwt_normalizes_email_input() {
        let user_id = Uuid::new_v4();
        let email = EmailAddress::from_str("TesT@eMaiL.Com").unwrap();
        let secret = JwtSecret::new("test-secret-that-is-long-enough-for-validation").unwrap();

        let token = Jwt::generate(user_id, email, secret.clone()).unwrap().jwt;
        let claims = Jwt::validate(&token, &secret).unwrap();
        assert_eq!(claims.email.to_string(), "test@email.com");
    }

    // ================================
    // JWT Validation Tests
    // ================================

    #[test]
    fn test_validate_jwt_success_returns_correct_claims() {
        let user_id = Uuid::new_v4();
        let email = EmailAddress::from_str("user@example.com").unwrap();
        let secret = JwtSecret::new("test-secret-that-is-long-enough-for-validation").unwrap();

        let token = Jwt::generate(user_id.clone(), email.clone(), secret.clone())
            .unwrap()
            .jwt;
        let claims = Jwt::validate(&token, &secret).unwrap();

        assert_eq!(claims.user_id, user_id);
        assert_eq!(claims.email, email);
    }

    #[test]
    fn test_validate_jwt_rejects_malformed_tokens() {
        let secret = JwtSecret::new("test-secret-that-is-long-enough-for-validation").unwrap();

        // Invalid JWT structure
        assert!(Jwt::new("invalid.token.here")
            .unwrap()
            .validate(&secret)
            .is_err());

        // Too many sections
        assert!(Jwt::new("token.with.too.many.sections")
            .unwrap()
            .validate(&secret)
            .is_err());

        // Too few sections
        assert!(Jwt::new("too.few").unwrap().validate(&secret).is_err());

        // Empty token
        assert!(Jwt::new("").is_err());
    }

    #[test]
    fn test_validate_jwt_rejects_wrong_secret() {
        let user_id = Uuid::new_v4();
        let email = EmailAddress::from_str("test@email.com").unwrap();
        let correct_secret =
            JwtSecret::new("correct-secret-that-is-long-enough-for-validation").unwrap();
        let wrong_secret =
            JwtSecret::new("wrong-secret-that-is-long-enough-for-validation").unwrap();

        let token = Jwt::generate(user_id, email, correct_secret).unwrap().jwt;
        let result = Jwt::validate(&token, &wrong_secret);
        assert!(result.is_err());
    }

    // ================================
    // JWT Claims Tests
    // ================================

    #[test]
    fn test_jwt_claims_have_correct_expiration_and_issued_at() {
        let user_id = Uuid::new_v4();
        let email = EmailAddress::from_str("test@email.com").unwrap();
        let secret = JwtSecret::new("test-secret-that-is-long-enough-for-validation").unwrap();

        let token = Jwt::generate(user_id, email, secret.clone()).unwrap().jwt;
        let claims = Jwt::validate(&token, &secret).unwrap();

        let now = chrono::Utc::now().timestamp() as usize;

        // Token should expire in the future
        assert!(claims.exp > now);

        // Token should be issued now or in the past
        assert!(claims.iat <= now);

        // Token should have 24-hour duration
        assert_eq!(claims.exp - claims.iat, 86400);
    }

    #[test]
    fn test_jwt_claims_match_input_data() {
        let user_id = Uuid::new_v4();
        let email = EmailAddress::from_str("specific@example.com").unwrap();
        let secret = JwtSecret::new("test-secret-that-is-long-enough-for-validation").unwrap();

        let token = Jwt::generate(user_id.clone(), email.clone(), secret.clone())
            .unwrap()
            .jwt;
        let claims = Jwt::validate(&token, &secret).unwrap();

        assert_eq!(claims.user_id, user_id);
        assert_eq!(claims.email, email);
    }

    // ================================
    // Integration Tests
    // ================================

    #[test]
    fn test_generate_and_validate_round_trip_with_multiple_user_ids() {
        let user_ids = vec![Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4()]; // Multiple random UUIDs
        let email = EmailAddress::from_str("test@email.com").unwrap();
        let secret = JwtSecret::new("test-secret-that-is-long-enough-for-validation").unwrap();

        for user_id in user_ids {
            let token = Jwt::generate(user_id.clone(), email.clone(), secret.clone())
                .unwrap()
                .jwt;
            let claims = Jwt::validate(&token, &secret).unwrap();
            assert_eq!(claims.user_id, user_id);
        }
    }

    #[test]
    fn test_complete_authentication_flow_round_trip() {
        // Simulate a complete auth flow: generate token, validate it, extract claims
        let original_user_id = Uuid::new_v4();
        let original_email = EmailAddress::from_str("auth@example.com").unwrap();
        let secret = JwtSecret::new("production-grade-secret-that-is-long-enough").unwrap();

        // Generate token (like during login)
        let token = Jwt::generate(
            original_user_id.clone(),
            original_email.clone(),
            secret.clone(),
        )
        .unwrap()
        .jwt;

        // Validate token (like during protected route access)
        let claims = Jwt::validate(&token, &secret).unwrap();

        // Verify all data survived the round trip
        assert_eq!(claims.user_id, original_user_id);
        assert_eq!(claims.email, original_email);

        let now = chrono::Utc::now().timestamp() as usize;
        assert!(claims.exp > now);
        assert!(claims.iat <= now);
    }
}
