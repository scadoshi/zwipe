//! Password reset completion handler.

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use zwipe_core::http::contracts::auth::HttpResetPassword;

use crate::domain::auth::requests::reset_password::{ResetPassword, ResetPasswordError};
use crate::domain::metrics::models::kinds::AuditAction;
use crate::inbound::http::{ApiError, AppState, To500};

impl From<ResetPasswordError> for ApiError {
    fn from(value: ResetPasswordError) -> Self {
        match value {
            ResetPasswordError::InvalidToken => {
                Self::Unauthorized("token not found or expired".to_string())
            }
            ResetPasswordError::InvalidPassword(msg) => Self::UnprocessableEntity(msg),
            ResetPasswordError::Database(e) => e.to_500(),
        }
    }
}

/// Completes the password reset flow using a one-time token.
///
/// Revokes all existing sessions after a successful reset.
///
/// # Errors
///
/// Returns 422 for a weak password, 401 for an unknown or expired token, and
/// 500 on a database failure.
pub async fn reset_password(
    State(state): State<AppState>,
    Json(body): Json<HttpResetPassword>,
) -> Result<StatusCode, ApiError> {
    // ResetPassword::new hashes with Argon2, so it runs on the blocking pool.
    let request = tokio::task::spawn_blocking(move || {
        ResetPassword::new(body.token, body.new_password.read())
    })
    .await
    .map_err(To500::to_500)??;
    let user_id = state
        .auth_service
        .reset_password(&request)
        .await
        .map_err(ApiError::from)?;

    let metrics = std::sync::Arc::clone(&state.metrics_service);
    tokio::spawn(async move {
        if let Err(e) = metrics
            .record_audit(user_id, AuditAction::PasswordChanged)
            .await
        {
            tracing::warn!(error = ?e, "metrics: audit password reset failed");
        }
    });

    Ok(StatusCode::OK)
}
