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

#[cfg(feature = "zerver")]
use crate::{
    domain::{
        auth::{
            models::access_token::{AccessToken, UserClaims},
            ports::AuthService,
        },
        card::ports::CardService,
        deck::ports::DeckService,
        health::ports::HealthService,
        user::ports::UserService,
    },
    inbound::http::AppState,
};
pub struct AuthenticatedUser {
    pub id: Uuid,
    pub email: EmailAddress,
}

impl From<UserClaims> for AuthenticatedUser {
    fn from(value: UserClaims) -> Self {
        Self {
            id: value.user_id,
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
        let TypedHeader(Authorization(bearer)) =
            TypedHeader::<Authorization<Bearer>>::from_request_parts(parts, state)
                .await
                .map_err(|_| StatusCode::BAD_REQUEST)?;
        let access_token = AccessToken::new(bearer.token()).map_err(|_| StatusCode::BAD_REQUEST)?;
        let claims = access_token
            .validate(state.auth_service.jwt_secret())
            .map_err(|_| StatusCode::UNAUTHORIZED)?;

        Ok(AuthenticatedUser::from(claims))
    }
}
