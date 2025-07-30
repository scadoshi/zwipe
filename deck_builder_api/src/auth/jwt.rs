use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct UserClaims {
    user_id: i32,
    email: String,
    exp: usize,
    iat: usize,
}

pub fn generate_jwt(user_id: i32, email: String) -> Result<String, Box<dyn std::error::Error>> {
    dotenvy::dotenv()?;

    let jwt_secret = std::env::var("JWT_SECRET")?.to_string();

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

pub fn validate_jwt(token: &str) -> Result<UserClaims, Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let jwt_secret = std::env::var("JWT_SECRET")?.to_string();

    let token_data = jsonwebtoken::decode::<UserClaims>(
        token,
        &DecodingKey::from_secret(jwt_secret.as_ref()),
        &Validation::default(),
    )?;

    Ok(token_data.claims)
}
