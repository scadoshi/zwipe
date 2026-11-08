//! Session revocation handler.

use axum::extract::State;
use axum::http::StatusCode;

use crate::domain::auth::ports::AuthService;
use crate::domain::auth::requests::revoke_sessions::{RevokeSessions, RevokeSessionsError};
use crate::domain::metrics::models::kinds::{AuditAction, EventKind};
use crate::domain::metrics::ports::MetricsService;
use crate::inbound::http::middleware::AuthenticatedUser;
use crate::inbound::http::{ApiError, AppState, To500};

impl From<RevokeSessionsError> for ApiError {
    fn from(value: RevokeSessionsError) -> Self {
        match value {
            RevokeSessionsError::Database(e) => e.to_500(),
        }
    }
}

/// Logs out the session the access token belongs to, named by its
/// `family_id` claim, and leaves the user's other sessions alone. A token
/// minted without the claim names no session, so its logout ends every
/// session the user holds.
///
/// # Errors
///
/// Returns 500 on a database failure.
pub async fn revoke_sessions(
    user: AuthenticatedUser,
    State(state): State<AppState>,
) -> Result<StatusCode, ApiError> {
    let request = RevokeSessions::family(user.id, user.family_id);
    end_sessions(&state, &request).await
}

/// Revokes what `request` names and records the logout.
///
/// # Errors
///
/// Returns 500 on a database failure.
pub(super) async fn end_sessions(
    state: &AppState,
    request: &RevokeSessions,
) -> Result<StatusCode, ApiError> {
    state
        .auth_service
        .revoke_sessions(request)
        .await
        .map_err(ApiError::from)?;

    let user_id = request.user_id;
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
