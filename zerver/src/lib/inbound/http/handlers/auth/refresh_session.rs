#[cfg(feature = "zerver")]
use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};

use crate::domain::auth::models::session::RefreshSession;
#[cfg(feature = "zerver")]
use crate::{
    domain::{
        auth::{
            models::session::{InvalidRefreshSession, RefreshSessionError, Session},
            ports::AuthService,
        },
        card::ports::CardService,
        deck::ports::DeckService,
        health::ports::HealthService,
        user::{models::get_user::GetUserError, ports::UserService},
    },
    inbound::http::{ApiError, AppState, Log500},
};

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
                tracing::info!("{}", RefreshSessionError::NotFound(u).to_string());
                Self::Unauthorized("invalid refresh token".to_string())
            }
            RefreshSessionError::Expired(u) => {
                tracing::info!("{}", RefreshSessionError::Expired(u).to_string());
                Self::Unauthorized("invalid refresh token".to_string())
            }
            RefreshSessionError::Revoked(u) => {
                tracing::warn!("{}", RefreshSessionError::Revoked(u).to_string());
                Self::Unauthorized("invalid refresh token".to_string())
            }
            RefreshSessionError::Forbidden(u) => {
                tracing::warn!("{}", RefreshSessionError::Forbidden(u).to_string());
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

#[derive(Debug, Deserialize, Serialize)]
pub struct HttpRefreshSession {
    user_id: String,
    refresh_token: String,
}

impl HttpRefreshSession {
    pub fn new(user_id: &str, refresh_token: &str) -> Self {
        Self {
            user_id: user_id.to_string(),
            refresh_token: refresh_token.to_string(),
        }
    }
}

impl From<RefreshSession> for HttpRefreshSession {
    fn from(value: RefreshSession) -> Self {
        Self {
            user_id: value.user_id.to_string(),
            refresh_token: value.refresh_token,
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
