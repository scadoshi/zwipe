//! JWT authentication middleware.

#[cfg(feature = "zerver")]
use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};
#[cfg(feature = "zerver")]
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use email_address::EmailAddress;
use uuid::Uuid;

use crate::domain::user::models::username::Username;
#[cfg(feature = "zerver")]
use crate::{
    domain::{
        auth::{models::access_token::UserClaims, ports::AuthService},
        card::ports::CardService,
        deck::ports::DeckService,
        health::ports::HealthService,
        user::ports::UserService,
    },
    inbound::http::AppState,
};

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
#[async_trait]
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

        use crate::domain::auth::models::access_token::Jwt;

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
