#[cfg(feature = "zerver")]
use axum::{extract::State, http::StatusCode};

#[cfg(feature = "zerver")]
use crate::{
    domain::{
        auth::requests::revoke_sessions::{RevokeSessions, RevokeSessionsError},
        metrics::models::kinds::{AuditAction, EventKind},
    },
    inbound::http::{ApiError, AppState, Log500, middleware::AuthenticatedUser},
};

#[cfg(feature = "zerver")]
impl From<RevokeSessionsError> for ApiError {
    fn from(value: RevokeSessionsError) -> Self {
        match value {
            RevokeSessionsError::Database(e) => e.log_500(),
        }
    }
}

/// Revokes all sessions for the authenticated user (logs out all devices).
#[cfg(feature = "zerver")]
pub async fn revoke_sessions(
    user: AuthenticatedUser,
    State(state): State<AppState>,
) -> Result<StatusCode, ApiError> {
    state
        .auth_service
        .revoke_sessions(&RevokeSessions::new(user.id))
        .await
        .map_err(ApiError::from)?;

    let user_id = user.id;
    let metrics = std::sync::Arc::clone(&state.metrics_service);
    // prime the debounce cache; logout is a deliberate user action, so it
    // counts as activity even though no authed request follows it
    state
        .last_active_cache
        .insert(user_id, std::time::Instant::now());
    tokio::spawn(async move {
        if let Err(e) = metrics.record_event(user_id, EventKind::Logout, None).await {
            tracing::warn!(error = ?e, "metrics: record logout event failed");
        }
        if let Err(e) = metrics.record_audit(user_id, AuditAction::Logout).await {
            tracing::warn!(error = ?e, "metrics: record logout audit failed");
        }
        if let Err(e) = metrics.touch_last_active(user_id).await {
            tracing::warn!(error = ?e, "metrics: touch_last_active on logout failed");
        }
    });

    Ok(StatusCode::NO_CONTENT)
}
