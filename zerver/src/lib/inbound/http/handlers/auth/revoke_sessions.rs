#[cfg(feature = "zerver")]
use axum::{extract::State, http::StatusCode};

#[cfg(feature = "zerver")]
use crate::{
    domain::{
        auth::{
            models::session::{RevokeSessions, RevokeSessionsError},
            ports::AuthService,
        },
        card::ports::CardService,
        deck::ports::DeckService,
        health::ports::HealthService,
        user::ports::UserService,
    },
    inbound::http::{middleware::AuthenticatedUser, ApiError, AppState, Log500},
};

#[cfg(feature = "zerver")]
impl From<RevokeSessionsError> for ApiError {
    fn from(value: RevokeSessionsError) -> Self {
        match value {
            RevokeSessionsError::Database(e) => e.log_500(),
        }
    }
}

#[cfg(feature = "zerver")]
pub async fn revoke_sessions<AS, US, HS, CS, DS>(
    user: AuthenticatedUser,
    State(state): State<AppState<AS, US, HS, CS, DS>>,
) -> Result<StatusCode, ApiError>
where
    AS: AuthService,
    US: UserService,
    HS: HealthService,
    CS: CardService,
    DS: DeckService,
{
    state
        .auth_service
        .revoke_sessions(&RevokeSessions::new(user.id))
        .await
        .map_err(ApiError::from)
        .map(|_| StatusCode::NO_CONTENT)
}
