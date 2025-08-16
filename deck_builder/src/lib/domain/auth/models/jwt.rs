use axum_extra::headers::authorization::Bearer;
use email_address::EmailAddress;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::domain::user::models::UserId;

//
//
//
//
//

#[derive(Debug, Clone, Error)]
pub enum JwtSecretError {
    #[error("Secret length must be 32+")]
    TooShort,
    #[error("Secret must be present")]
    MissingSecret,
}

#[derive(Debug, Clone, Error)]
pub enum JwtError {
    #[error("Token must be present")]
    MissingToken,
    #[error("Invalid token format")]
    InvalidFormat,
}

//
//
//
//
//

#[derive(Debug, Clone)]
pub struct JwtSecret(String);

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

#[derive(Debug, Clone)]
pub struct Jwt(String);

impl Jwt {}

//
//
//
//
//

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct UserClaims {
    pub user_id: i32,
    pub email: String,
    pub exp: usize,
    pub iat: usize,
}

//
//
//
//
//

impl Jwt {
    pub fn generate(
        user_id: UserId,
        email: EmailAddress,
        secret: JwtSecret,
    ) -> Result<Self, jsonwebtoken::errors::Error> {
        let user_claims = UserClaims {
            user_id,
            email,
            exp: (chrono::Utc::now().timestamp() + 86400) as usize,
            iat: chrono::Utc::now().timestamp() as usize,
        };
        jsonwebtoken::encode(
            &Header::default(),
            &user_claims,
            &EncodingKey::from_secret(secret.0.as_ref()),
        )?
    }

    pub fn new(bearer: Bearer) -> Result<Self, JwtError> {
        if bearer.is_empty() {
            return Err(JwtError::MissingToken);
        }
        if bearer.split(".").count() != 3 {
            return Err(JwtError::InvalidFormat);
        }
        Ok(Self(bearer.to_string()))
    }

    pub fn validate(&self, secret: JwtSecret) -> Result<UserClaims, jsonwebtoken::errors::Error> {
        let token_data = jsonwebtoken::decode::<UserClaims>(
            self,
            &DecodingKey::from_secret(secret.0.as_ref()),
            &Validation::default(),
        )?;
        Ok(token_data.claims)
    }
}

//
//
//
//
//

#[cfg(test)]
mod tests {
    use super::*;

    // ================================
    // JWT Generation Tests
    // ================================

    #[test]
    fn test_generate_jwt_success_creates_valid_tokens() {
        let result = Jwt::generate(1, "test@email.com".to_string(), "test-secret");
        assert!(result.is_ok());
        let token = result.unwrap();
        assert!(!token.is_empty());
        assert_eq!(token.split('.').count(), 3); // JWT has 3 parts
    }

    #[test]
    fn test_generate_jwt_produces_consistent_results() {
        let token1 = Jwt::generate(1, "test@email.com".to_string(), "test-secret").unwrap();
        let token2 = Jwt::generate(1, "test@email.com".to_string(), "test-secret").unwrap();
        assert_eq!(token1, token2);
    }

    #[test]
    fn test_generate_jwt_produces_unique_tokens_for_different_users() {
        let token1 = Jwt::generate(1, "user1@email.com".to_string(), "test-secret").unwrap();
        let token2 = Jwt::generate(2, "user2@email.com".to_string(), "test-secret").unwrap();
        assert_ne!(token1, token2);
    }

    #[test]
    fn test_generate_jwt_produces_unique_tokens_for_different_secrets() {
        let token1 = Jwt::generate(1, "test@email.com".to_string(), "secret-1").unwrap();
        let token2 = Jwt::generate(1, "test@email.com".to_string(), "secret-2").unwrap();
        assert_ne!(token1, token2);
    }

    #[test]
    fn test_generate_jwt_normalizes_email_input() {
        let messy_email = " TesT@eMaiL.Com   ".to_string();
        let token = Jwt::generate(1, messy_email, "test-secret").unwrap();
        let claims = Jwt::validate(&token, "test-secret").unwrap();
        assert_eq!(claims.email, "test@email.com");
    }

    #[test]
    fn test_generate_jwt_rejects_invalid_user_ids() {
        // Negative user ID
        assert!(Jwt::generate(-1, "test@email.com".to_string(), "test-secret").is_err());

        // Zero user ID
        assert!(Jwt::generate(0, "test@email.com".to_string(), "test-secret").is_err());

        // User ID above maximum
        assert!(Jwt::generate(100_000_001, "test@email.com".to_string(), "test-secret").is_err());
    }

    #[test]
    fn test_generate_jwt_rejects_empty_email() {
        let result = Jwt::generate(1, "".to_string(), "test-secret");
        assert!(result.is_err());
    }

    #[test]
    fn test_generate_jwt_rejects_empty_secret() {
        let result = Jwt::generate(1, "test@email.com".to_string(), "");
        assert!(result.is_err());
    }

    // ================================
    // JWT Validation Tests
    // ================================

    #[test]
    fn test_validate_jwt_success_returns_correct_claims() {
        let token = Jwt::generate(42, "user@example.com".to_string(), "test-secret").unwrap();
        let claims = Jwt::validate(&token, "test-secret").unwrap();

        assert_eq!(claims.user_id, 42);
        assert_eq!(claims.email, "user@example.com");
    }

    #[test]
    fn test_validate_jwt_rejects_malformed_tokens() {
        // Invalid JWT structure
        assert!(Jwt::validate("invalid.token.here", "test-secret").is_err());

        // Too many sections
        assert!(Jwt::validate("token.with.too.many.sections", "test-secret").is_err());

        // Too few sections
        assert!(Jwt::validate("too.few", "test-secret").is_err());

        // Empty token
        assert!(Jwt::validate("", "test-secret").is_err());
    }

    #[test]
    fn test_validate_jwt_rejects_wrong_secret() {
        let token = Jwt::generate(1, "test@email.com".to_string(), "correct-secret").unwrap();
        let result = Jwt::validate(&token, "wrong-secret");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_jwt_rejects_empty_secret() {
        let token = Jwt::generate(1, "test@email.com".to_string(), "valid-secret").unwrap();
        let result = Jwt::validate(&token, "");
        assert!(result.is_err());
    }

    // ================================
    // JWT Claims Tests
    // ================================

    #[test]
    fn test_jwt_claims_have_correct_expiration_and_issued_at() {
        let token = Jwt::generate(1, "test@email.com".to_string(), "test-secret").unwrap();
        let claims = Jwt::validate(&token, "test-secret").unwrap();

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
        let user_id = 12345;
        let email = "specific@example.com".to_string();
        let token = Jwt::generate(user_id, email.clone(), "test-secret").unwrap();
        let claims = Jwt::validate(&token, "test-secret").unwrap();

        assert_eq!(claims.user_id, user_id);
        assert_eq!(claims.email, email);
    }

    // ================================
    // Integration Tests
    // ================================

    #[test]
    fn test_generate_and_validate_round_trip_with_edge_case_user_ids() {
        let edge_case_user_ids = vec![1, 99_999_999]; // Min and near-max valid IDs

        for user_id in edge_case_user_ids {
            let token =
                Jwt::generate(user_id, "test@email.com".to_string(), "test-secret").unwrap();
            let claims = Jwt::validate(&token, "test-secret").unwrap();
            assert_eq!(claims.user_id, user_id);
        }
    }

    #[test]
    fn test_complete_authentication_flow_round_trip() {
        // Simulate a complete auth flow: generate token, validate it, extract claims
        let original_user_id = 1337;
        let original_email = "auth@example.com".to_string();
        let secret = "production-grade-secret";

        // Generate token (like during login)
        let token = Jwt::generate(original_user_id, original_email.clone(), secret).unwrap();

        // Validate token (like during protected route access)
        let claims = Jwt::validate(&token, secret).unwrap();

        // Verify all data survived the round trip
        assert_eq!(claims.user_id, original_user_id);
        assert_eq!(claims.email, original_email);

        let now = chrono::Utc::now().timestamp() as usize;
        assert!(claims.exp > now);
        assert!(claims.iat <= now);
    }
}
