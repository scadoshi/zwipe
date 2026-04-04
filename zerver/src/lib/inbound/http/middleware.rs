//! JWT authentication middleware.

use zwipe_core::domain::{
    auth::models::access_token::{Jwt, UserClaims},
    user::username::Username,
};
#[cfg(feature = "zerver")]
use crate::{
    domain::{
        auth::{
            models::access_token::{JwtSecret, JwtValidate},
            ports::AuthService,
        },
        card::ports::CardService,
        deck::ports::DeckService,
        health::ports::HealthService,
        user::ports::UserService,
    },
    inbound::http::AppState,
};
#[cfg(feature = "zerver")]
use axum::http::header::AUTHORIZATION;
#[cfg(feature = "zerver")]
use axum::{
    extract::FromRequestParts,
    http::{StatusCode, request::Parts},
};
#[cfg(feature = "zerver")]
use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
};
use email_address::EmailAddress;
#[cfg(feature = "zerver")]
use std::str::FromStr;
#[cfg(feature = "zerver")]
use tower_governor::{GovernorError, key_extractor::KeyExtractor};
use uuid::Uuid;

/// Axum extractor that enforces JWT authentication.
///
/// Including this in a handler signature means the route requires a valid Bearer token.
/// Extraction flow: `Authorization: Bearer <token>` → parse JWT → validate signature
/// → extract claims.
///
/// Rejects with `400 Bad Request` if the header is missing or malformed,
/// `401 Unauthorized` if the signature is invalid.
pub struct AuthenticatedUser {
    /// User ID from JWT claims.
    pub id: Uuid,
    /// Username from JWT claims.
    pub username: Username,
    /// Email from JWT claims.
    pub email: EmailAddress,
}

/// Rate-limit key extractor that keys by authenticated user ID from the JWT.
///
/// Used on private routes so each user gets their own rate limit bucket
/// regardless of IP address. Falls back to `UnableToExtractKey` for
/// missing or invalid tokens — the auth middleware rejects those downstream.
#[cfg(feature = "zerver")]
#[derive(Debug, Clone)]
pub struct UserIdKeyExtractor {
    jwt_secret: JwtSecret,
}

#[cfg(feature = "zerver")]
impl UserIdKeyExtractor {
    /// Creates a new extractor with the given JWT secret for token validation.
    pub fn new(jwt_secret: JwtSecret) -> Self {
        Self { jwt_secret }
    }
}

#[cfg(feature = "zerver")]
impl KeyExtractor for UserIdKeyExtractor {
    type Key = Uuid;
    fn extract<T>(
        &self,
        req: &axum::http::Request<T>,
    ) -> Result<Self::Key, tower_governor::errors::GovernorError> {
        let token = req
            .headers()
            .get(AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .ok_or(GovernorError::UnableToExtractKey)?;

        let jwt = Jwt::from_str(token).map_err(|_| GovernorError::UnableToExtractKey)?;
        let claims = jwt
            .validate(&self.jwt_secret)
            .map_err(|_| GovernorError::UnableToExtractKey)?;

        Ok(claims.user_id)
    }
}

impl From<UserClaims> for AuthenticatedUser {
    fn from(value: UserClaims) -> Self {
        Self {
            id: value.user_id,
            username: value.username,
            email: value.email,
        }
    }
}

#[cfg(feature = "zerver")]
impl<AS, US, HS, CS, DS> FromRequestParts<AppState<AS, US, HS, CS, DS>> for AuthenticatedUser
where
    AS: AuthService,
    US: UserService,
    HS: HealthService,
    CS: CardService,
    DS: DeckService,
{
    type Rejection = StatusCode;
    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState<AS, US, HS, CS, DS>,
    ) -> Result<Self, Self::Rejection> {
        use std::str::FromStr;

        use zwipe_core::domain::auth::models::access_token::Jwt;

        let TypedHeader(Authorization(bearer)) =
            TypedHeader::<Authorization<Bearer>>::from_request_parts(parts, state)
                .await
                .map_err(|_| StatusCode::BAD_REQUEST)?;
        let jwt = Jwt::from_str(bearer.token()).map_err(|_| StatusCode::BAD_REQUEST)?;
        let claims = jwt
            .validate(state.auth_service.jwt_secret())
            .map_err(|_| StatusCode::UNAUTHORIZED)?;

        Ok(AuthenticatedUser::from(claims))
    }
}
