use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct UserClaims {
    user_id: i32,
    email: String,
    exp: usize,
    iat: usize,
}

#[derive(Debug, Clone)]
pub struct JwtConfig {
    pub secret: String,
}

impl JwtConfig {
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        let secret = std::env::var("JWT_SECRET")?;
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
) -> Result<String, Box<dyn std::error::Error>> {
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

pub fn validate_jwt(
    token: &str,
    jwt_secret: &str,
) -> Result<UserClaims, Box<dyn std::error::Error>> {
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
    #[test]
    fn basic_generation() {
        let (jwt_1_result, jwt_2_result) = (
            generate_jwt(1, "test1@email.com".to_string(), "abc-123"),
            generate_jwt(2, "test2@email.com".to_string(), "abc-123"),
        );
        // success
        assert!(jwt_1_result.is_ok() && jwt_2_result.is_ok());
        let (jwt_1, jwt_2) = (jwt_1_result.unwrap(), jwt_2_result.unwrap());
        // consistent results
        assert_eq!(
            jwt_1,
            generate_jwt(1, "test1@email.com".to_string(), "abc-123").unwrap()
        );
        // different results
        assert_ne!(jwt_1, jwt_2);
    }

    #[test]
    fn generation_errors() {
        // bad ids
        assert!(generate_jwt(-1, "test@email.com".to_string(), "abc-123").is_err());
        assert!(generate_jwt(0, "test@email.com".to_string(), "abc-123").is_err());
        assert!(generate_jwt(100000001, "test1@email.com".to_string(), "abc-123").is_err());
        // no email
        assert!(generate_jwt(1, "".to_string(), "abc-123").is_err());
    }

    #[test]
    fn generation_normalization() {
        let messy_email = " TesT@eMaiL.Com   ".to_string();
        let token = generate_jwt(1, messy_email, "abc-123").unwrap();
        let claims = validate_jwt(&token, "abc-123").unwrap();
        assert_eq!(claims.email, "test@email.com");
    }

    #[test]
    fn basic_validation() {
        let jwt_1 = generate_jwt(1, "test1@email.com".to_string(), "abc-123").unwrap();
        // validation
        assert_eq!(
            UserClaims {
                user_id: 1,
                email: "test1@email.com".to_string(),
                exp: (chrono::Utc::now().timestamp() + 86400) as usize,
                iat: chrono::Utc::now().timestamp() as usize,
            },
            validate_jwt(&jwt_1, "abc-123").unwrap()
        );
    }
    #[test]
    fn validation_errors() {
        // bad token
        assert!(validate_jwt("invalid.token.here", "abc-123").is_err());
        assert!(validate_jwt("token.with.too.many.sections", "abc-123").is_err());
        assert!(validate_jwt("too.few", "abc-123").is_err());
        assert!(validate_jwt("", "abc-123").is_err());
    }

    #[test]
    fn secret_related() {
        // different secrets
        assert_ne!(
            generate_jwt(1, "test".to_string(), "abc-123").unwrap(),
            generate_jwt(1, "test".to_string(), "def-456").unwrap()
        );
        // empty secret
        assert!(generate_jwt(1, "test".to_string(), &"").is_err());
    }
}
