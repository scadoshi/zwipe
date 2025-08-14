use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::error::Error as StdError;
use thiserror::Error;
use tracing::info;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct UserClaims {
    pub user_id: i32,
    pub email: String,
    pub exp: usize,
    pub iat: usize,
}

#[derive(Debug, Clone)]
pub struct JwtConfig {
    pub secret: String,
}

impl JwtConfig {
    pub fn from_env() -> Result<Self, Box<dyn StdError>> {
        let secret = std::env::var("JWT_SECRET")?;
        info!("Extracted JSON web token secret from environment");
        Ok(JwtConfig { secret })
    }
}

#[derive(Error, Debug)]
pub enum JwtGenerationError {
    #[error("Invalid user ID: {0}")]
    InvalidUserId(i32),
    #[error("Invalid email: {0}")]
    InvalidEmail(String),
    #[error("Missing or empty secret")]
    MissingSecret,
}

pub fn generate_jwt(
    user_id: i32,
    email: String,
    jwt_secret: &str,
) -> Result<String, Box<dyn StdError>> {
    if user_id < 1 || user_id > 100_000_000 {
        return Err(Box::new(JwtGenerationError::InvalidUserId(user_id)));
    }

    if email.is_empty() {
        return Err(Box::new(JwtGenerationError::InvalidEmail(email)));
    }

    if jwt_secret.is_empty() {
        return Err(Box::new(JwtGenerationError::MissingSecret));
    }

    let normalized_email = email.trim().to_lowercase();

    let user_claims = UserClaims {
        user_id,
        email: normalized_email,
        exp: (chrono::Utc::now().timestamp() + 86400) as usize,
        iat: chrono::Utc::now().timestamp() as usize,
    };

    let token = jsonwebtoken::encode(
        &Header::default(),
        &user_claims,
        &EncodingKey::from_secret(jwt_secret.as_ref()),
    )?;

    Ok(token)
}

#[derive(Error, Debug)]
pub enum JwtValidationError {
    #[error("Invalid token: {0}")]
    InvalidToken(String),
    #[error("Given secret was empty. Secret is required.")]
    MissingSecret,
}

pub fn validate_jwt(token: &str, jwt_secret: &str) -> Result<UserClaims, Box<dyn StdError>> {
    if token.is_empty() || token.split(".").count() != 3 {
        return Err(Box::new(JwtValidationError::InvalidToken(
            token.to_string(),
        )));
    }

    if jwt_secret.is_empty() {
        return Err(Box::new(JwtValidationError::MissingSecret));
    }

    let token_data = jsonwebtoken::decode::<UserClaims>(
        token,
        &DecodingKey::from_secret(jwt_secret.as_ref()),
        &Validation::default(),
    )?;

    Ok(token_data.claims)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ================================
    // JWT Generation Tests
    // ================================

    #[test]
    fn test_generate_jwt_success_creates_valid_tokens() {
        let result = generate_jwt(1, "test@email.com".to_string(), "test-secret");
        assert!(result.is_ok());
        let token = result.unwrap();
        assert!(!token.is_empty());
        assert_eq!(token.split('.').count(), 3); // JWT has 3 parts
    }

    #[test]
    fn test_generate_jwt_produces_consistent_results() {
        let token1 = generate_jwt(1, "test@email.com".to_string(), "test-secret").unwrap();
        let token2 = generate_jwt(1, "test@email.com".to_string(), "test-secret").unwrap();
        assert_eq!(token1, token2);
    }

    #[test]
    fn test_generate_jwt_produces_unique_tokens_for_different_users() {
        let token1 = generate_jwt(1, "user1@email.com".to_string(), "test-secret").unwrap();
        let token2 = generate_jwt(2, "user2@email.com".to_string(), "test-secret").unwrap();
        assert_ne!(token1, token2);
    }

    #[test]
    fn test_generate_jwt_produces_unique_tokens_for_different_secrets() {
        let token1 = generate_jwt(1, "test@email.com".to_string(), "secret-1").unwrap();
        let token2 = generate_jwt(1, "test@email.com".to_string(), "secret-2").unwrap();
        assert_ne!(token1, token2);
    }

    #[test]
    fn test_generate_jwt_normalizes_email_input() {
        let messy_email = " TesT@eMaiL.Com   ".to_string();
        let token = generate_jwt(1, messy_email, "test-secret").unwrap();
        let claims = validate_jwt(&token, "test-secret").unwrap();
        assert_eq!(claims.email, "test@email.com");
    }

    #[test]
    fn test_generate_jwt_rejects_invalid_user_ids() {
        // Negative user ID
        assert!(generate_jwt(-1, "test@email.com".to_string(), "test-secret").is_err());

        // Zero user ID
        assert!(generate_jwt(0, "test@email.com".to_string(), "test-secret").is_err());

        // User ID above maximum
        assert!(generate_jwt(100_000_001, "test@email.com".to_string(), "test-secret").is_err());
    }

    #[test]
    fn test_generate_jwt_rejects_empty_email() {
        let result = generate_jwt(1, "".to_string(), "test-secret");
        assert!(result.is_err());
    }

    #[test]
    fn test_generate_jwt_rejects_empty_secret() {
        let result = generate_jwt(1, "test@email.com".to_string(), "");
        assert!(result.is_err());
    }

    // ================================
    // JWT Validation Tests
    // ================================

    #[test]
    fn test_validate_jwt_success_returns_correct_claims() {
        let token = generate_jwt(42, "user@example.com".to_string(), "test-secret").unwrap();
        let claims = validate_jwt(&token, "test-secret").unwrap();

        assert_eq!(claims.user_id, 42);
        assert_eq!(claims.email, "user@example.com");
    }

    #[test]
    fn test_validate_jwt_rejects_malformed_tokens() {
        // Invalid JWT structure
        assert!(validate_jwt("invalid.token.here", "test-secret").is_err());

        // Too many sections
        assert!(validate_jwt("token.with.too.many.sections", "test-secret").is_err());

        // Too few sections
        assert!(validate_jwt("too.few", "test-secret").is_err());

        // Empty token
        assert!(validate_jwt("", "test-secret").is_err());
    }

    #[test]
    fn test_validate_jwt_rejects_wrong_secret() {
        let token = generate_jwt(1, "test@email.com".to_string(), "correct-secret").unwrap();
        let result = validate_jwt(&token, "wrong-secret");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_jwt_rejects_empty_secret() {
        let token = generate_jwt(1, "test@email.com".to_string(), "valid-secret").unwrap();
        let result = validate_jwt(&token, "");
        assert!(result.is_err());
    }

    // ================================
    // JWT Claims Tests
    // ================================

    #[test]
    fn test_jwt_claims_have_correct_expiration_and_issued_at() {
        let token = generate_jwt(1, "test@email.com".to_string(), "test-secret").unwrap();
        let claims = validate_jwt(&token, "test-secret").unwrap();

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
        let token = generate_jwt(user_id, email.clone(), "test-secret").unwrap();
        let claims = validate_jwt(&token, "test-secret").unwrap();

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
            let token = generate_jwt(user_id, "test@email.com".to_string(), "test-secret").unwrap();
            let claims = validate_jwt(&token, "test-secret").unwrap();
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
        let token = generate_jwt(original_user_id, original_email.clone(), secret).unwrap();

        // Validate token (like during protected route access)
        let claims = validate_jwt(&token, secret).unwrap();

        // Verify all data survived the round trip
        assert_eq!(claims.user_id, original_user_id);
        assert_eq!(claims.email, original_email);

        let now = chrono::Utc::now().timestamp() as usize;
        assert!(claims.exp > now);
        assert!(claims.iat <= now);
    }
}
