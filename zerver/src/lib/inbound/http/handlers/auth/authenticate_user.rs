#[cfg(feature = "zerver")]
use axum::{Json, extract::State, http::StatusCode};
#[cfg(feature = "zerver")]
use zwipe_core::domain::auth::models::session::Session;
#[cfg(feature = "zerver")]
use zwipe_core::http::contracts::auth::HttpAuthenticateUser;

#[cfg(feature = "zerver")]
use crate::domain::auth::requests::authenticate_user::AuthenticateUser;
#[cfg(feature = "zerver")]
use crate::{
    domain::{
        auth::requests::authenticate_user::{AuthenticateUserError, InvalidAuthenticateUser},
        metrics::models::kinds::{AuditAction, EventKind},
    },
    inbound::http::{ApiError, AppState, Log500},
};

#[cfg(feature = "zerver")]
impl From<AuthenticateUserError> for ApiError {
    fn from(value: AuthenticateUserError) -> Self {
        match value {
            AuthenticateUserError::UserNotFound | AuthenticateUserError::InvalidPassword => {
                Self::Unauthorized("invalid credentials".to_string())
            }
            AuthenticateUserError::Database(e) => e.log_500(),
            AuthenticateUserError::UserFromDb(e) => e.log_500(),
            AuthenticateUserError::FailedToVerify(e) => e.log_500(),
            AuthenticateUserError::FailedAccessToken(e) => e.log_500(),
            AuthenticateUserError::CreateSessionError(e) => ApiError::from(e),
            AuthenticateUserError::AccountLocked => {
                Self::TooManyRequests("account temporarily locked".to_string())
            }
        }
    }
}

#[cfg(feature = "zerver")]
impl From<InvalidAuthenticateUser> for ApiError {
    fn from(value: InvalidAuthenticateUser) -> Self {
        match value {
            InvalidAuthenticateUser::MissingIdentifier
            | InvalidAuthenticateUser::MissingPassword => {
                Self::UnprocessableEntity("invalid credentials".to_string())
            }
        }
    }
}

#[cfg(feature = "zerver")]
impl TryFrom<HttpAuthenticateUser> for AuthenticateUser {
    type Error = InvalidAuthenticateUser;
    fn try_from(value: HttpAuthenticateUser) -> Result<Self, Self::Error> {
        AuthenticateUser::new(&value.identifier, &value.password)
    }
}

/// Authenticates a user by email or username and returns a session.
#[cfg(feature = "zerver")]
pub async fn authenticate_user(
    State(state): State<AppState>,
    Json(body): Json<HttpAuthenticateUser>,
) -> Result<(StatusCode, Json<Session>), ApiError> {
    let mut request = AuthenticateUser::new(&body.identifier, &body.password)?;
    request.platform = body.platform;

    let session = state
        .auth_service
        .authenticate_user(&request)
        .await
        .map_err(ApiError::from)?;

    let user_id = session.user.id;
    let metrics = std::sync::Arc::clone(&state.metrics_service);
    // prime the debounce cache so the first authed request after login
    // doesn't immediately bump last_active_at a second time
    state
        .last_active_cache
        .insert(user_id, std::time::Instant::now());
    tokio::spawn(async move {
        if let Err(e) = metrics.record_event(user_id, EventKind::Login, None).await {
            tracing::warn!(error = ?e, "metrics: record login event failed");
        }
        if let Err(e) = metrics.record_audit(user_id, AuditAction::Login).await {
            tracing::warn!(error = ?e, "metrics: record login audit failed");
        }
        if let Err(e) = metrics.touch_last_active(user_id).await {
            tracing::warn!(error = ?e, "metrics: touch_last_active on login failed");
        }
    });

    Ok((StatusCode::OK, session.into()))
}
