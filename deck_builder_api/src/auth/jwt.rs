use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct UserClaims {
    user_id: i32,
    email: String,
    exp: usize,
    iat: usize,
}

pub fn generate_jwt(user_id: i32, email: String) -> Result<String, jsonwebtoken::errors::Error> {
    dotenvy::dotenv().ok();

    let jwt_secret = std::env::var("JWT_SECRET")
        .expect("JWT_SECRET not found")
        .to_string();

    let user_claims = UserClaims {
        user_id,
        email,
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

pub fn validate_jwt(token: &str) -> Result<UserClaims, jsonwebtoken::errors::Error> {
    dotenvy::dotenv().ok();

    let jwt_secret = std::env::var("JWT_SECRET")
        .expect("JWT_SECRET not found")
        .to_string();

    let token_data = jsonwebtoken::decode::<UserClaims>(
        token,
        &DecodingKey::from_secret(jwt_secret.as_ref()),
        &Validation::default(),
    )?;

    Ok(token_data.claims)
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    pub fn test_jwt_round_trip() {
        std::env::set_var("JWT_SECRET", "test_secret_key");
        let token = generate_jwt(1, "test@email.com".to_string()).unwrap();
        println!("Generated token: {}", token);
        let claims = validate_jwt(&token).unwrap();
        assert_eq!(claims.user_id, 1);
        assert_eq!(claims.email, "test@email.com");
        println!("Claims: {:?}", claims);
    }
}
