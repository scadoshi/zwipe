use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use email_address::EmailAddress;
use uuid::Uuid;

use crate::{
    domain::{
        auth::{
            models::jwt::{Jwt, UserClaims},
            ports::AuthService,
        },
        card::ports::CardService,
        health::ports::HealthService,
        user::ports::UserService,
    },
    inbound::http::AppState,
};
pub struct AuthenticatedUser {
    pub user_id: Uuid,
    pub email: EmailAddress,
}

impl From<UserClaims> for AuthenticatedUser {
    fn from(value: UserClaims) -> Self {
        Self {
            user_id: value.user_id,
            email: value.email,
        }
    }
}

#[async_trait]
impl<AS, US, HS, CS> FromRequestParts<AppState<AS, US, HS, CS>> for AuthenticatedUser
where
    AS: AuthService,
    US: UserService,
    HS: HealthService,
    CS: CardService,
{
    type Rejection = StatusCode;
    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState<AS, US, HS, CS>,
    ) -> Result<Self, Self::Rejection> {
        let TypedHeader(Authorization(bearer)) =
            TypedHeader::<Authorization<Bearer>>::from_request_parts(parts, state)
                .await
                .map_err(|_| StatusCode::BAD_REQUEST)?;
        let jwt = Jwt::new(bearer.token()).map_err(|_| StatusCode::BAD_REQUEST)?;
        let claims = jwt
            .validate(state.auth_service.jwt_secret())
            .map_err(|_| StatusCode::UNAUTHORIZED)?;

        Ok(AuthenticatedUser::from(claims))
    }
}

#[cfg(test)]
mod tests {
    // Test valid JWT extraction
    // Test missing Authorization header (400)
    // Test malformed Bearer token (400)
    // Test invalid JWT signature (401)
    // Test expired JWT (401)
}
