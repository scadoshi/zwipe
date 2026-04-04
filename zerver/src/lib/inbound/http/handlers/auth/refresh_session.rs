#[cfg(feature = "zerver")]
use crate::domain::auth::requests::refresh_session::RefreshSession;
use zwipe_core::domain::auth::models::session::Session;
#[cfg(feature = "zerver")]
use crate::{
    domain::{
        auth::{
            ports::AuthService,
            requests::refresh_session::{InvalidRefreshSession, RefreshSessionError},
        },
        card::ports::CardService,
        deck::ports::DeckService,
        health::ports::HealthService,
        user::{models::get_user::GetUserError, ports::UserService},
    },
    inbound::http::{ApiError, AppState, Log500},
};
#[cfg(feature = "zerver")]
use axum::{extract::State, http::StatusCode, Json};
pub use zwipe_core::http::contracts::auth::HttpRefreshSession;

#[cfg(feature = "zerver")]
impl From<RefreshSessionError> for ApiError {
    fn from(value: RefreshSessionError) -> Self {
        match value {
            RefreshSessionError::CreateSessionError(e) => ApiError::from(e),
            RefreshSessionError::Database(e) => e.log_500(),
            RefreshSessionError::GetUserError(GetUserError::NotFound) => {
                Self::Unauthorized("invalid refresh token".to_string())
            }
            RefreshSessionError::GetUserError(e) => e.log_500(),
            RefreshSessionError::InvalidJwt(e) => e.log_500(),
            RefreshSessionError::EnforceSessionMaximumError(e) => ApiError::from(e),
            RefreshSessionError::NotFound(u) => {
                tracing::warn!(event = "token_refresh_failure", reason = "not_found", user_id = %u);
                Self::Unauthorized("invalid refresh token".to_string())
            }
            RefreshSessionError::Expired(u) => {
                tracing::warn!(event = "token_refresh_failure", reason = "expired", user_id = %u);
                Self::Unauthorized("invalid refresh token".to_string())
            }
            RefreshSessionError::Revoked(u) => {
                tracing::warn!(event = "token_refresh_failure", reason = "revoked", user_id = %u);
                Self::Unauthorized("invalid refresh token".to_string())
            }
            RefreshSessionError::Forbidden(u) => {
                tracing::warn!(event = "token_refresh_failure", reason = "forbidden", user_id = %u);
                Self::Forbidden("invalid refresh token".to_string())
            }
        }
    }
}

#[cfg(feature = "zerver")]
impl From<InvalidRefreshSession> for ApiError {
    fn from(value: InvalidRefreshSession) -> Self {
        match value {
            InvalidRefreshSession::UserId(_) => {
                Self::UnprocessableEntity("invalid user id".to_string())
            }
        }
    }
}

#[cfg(feature = "zerver")]
impl TryFrom<HttpRefreshSession> for RefreshSession {
    type Error = InvalidRefreshSession;
    fn try_from(value: HttpRefreshSession) -> Result<Self, Self::Error> {
        Self::new(&value.user_id, &value.refresh_token)
    }
}

/// Rotates a refresh token, consuming the old one and issuing a new session.
#[cfg(feature = "zerver")]
pub async fn refresh_session<AS, US, HS, CS, DS>(
    State(state): State<AppState<AS, US, HS, CS, DS>>,
    Json(body): Json<HttpRefreshSession>,
) -> Result<(StatusCode, Json<Session>), ApiError>
where
    AS: AuthService,
    US: UserService,
    HS: HealthService,
    CS: CardService,
    DS: DeckService,
{
    let request = RefreshSession::new(&body.user_id, &body.refresh_token)?;

    state
        .auth_service
        .refresh_session(&request)
        .await
        .map_err(ApiError::from)
        .map(|response| (StatusCode::OK, response.into()))
}
