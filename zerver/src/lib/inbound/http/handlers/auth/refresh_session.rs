#[cfg(feature = "zerver")]
use crate::domain::auth::requests::refresh_session::RefreshSession;
#[cfg(feature = "zerver")]
use crate::{
    domain::{
        auth::requests::refresh_session::{InvalidRefreshSession, RefreshSessionError},
        metrics::models::kinds::{AuditAction, EventKind},
        user::models::get_user::GetUserError,
    },
    inbound::http::{ApiError, AppState, Log500},
};
#[cfg(feature = "zerver")]
use axum::{Json, extract::State, http::StatusCode};
#[cfg(feature = "zerver")]
use zwipe_core::domain::auth::models::session::Session;
#[cfg(feature = "zerver")]
use zwipe_core::http::contracts::auth::HttpRefreshSession;

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
        let mut request = Self::new(&value.user_id, &value.refresh_token)?;
        request.client_version = value.client_version;
        Ok(request)
    }
}

/// Rotates a refresh token, consuming the old one and issuing a new session.
#[cfg(feature = "zerver")]
pub async fn refresh_session(
    State(state): State<AppState>,
    Json(body): Json<HttpRefreshSession>,
) -> Result<(StatusCode, Json<Session>), ApiError> {
    let mut request = RefreshSession::new(&body.user_id, &body.refresh_token)?;
    request.client_version = body.client_version;

    let session = state
        .auth_service
        .refresh_session(&request)
        .await
        .map_err(ApiError::from)?;

    let user_id = session.user.id;
    let metrics = std::sync::Arc::clone(&state.metrics_service);
    // prime the debounce cache so the first authed request after refresh
    // doesn't immediately bump last_active_at a second time
    state
        .last_active_cache
        .insert(user_id, std::time::Instant::now());
    tokio::spawn(async move {
        if let Err(e) = metrics
            .record_event(user_id, EventKind::Refresh, None)
            .await
        {
            tracing::warn!(error = ?e, "metrics: record refresh event failed");
        }
        if let Err(e) = metrics.record_audit(user_id, AuditAction::Refresh).await {
            tracing::warn!(error = ?e, "metrics: record refresh audit failed");
        }
        if let Err(e) = metrics.touch_last_active(user_id).await {
            tracing::warn!(error = ?e, "metrics: touch_last_active on refresh failed");
        }
    });

    Ok((StatusCode::OK, session.into()))
}
