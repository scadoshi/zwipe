#[cfg(feature = "zerver")]
use axum::{Json, extract::State, http::StatusCode};
#[cfg(feature = "zerver")]
use zwipe_core::http::contracts::auth::HttpChangePassword;

#[cfg(feature = "zerver")]
use crate::{
    domain::{
        auth::requests::change_password::{
            ChangePassword, ChangePasswordError, InvalidChangePassword,
        },
        metrics::models::kinds::AuditAction,
    },
    inbound::http::{ApiError, AppState, Log500, middleware::AuthenticatedUser},
};

#[cfg(feature = "zerver")]
impl From<ChangePasswordError> for ApiError {
    fn from(value: ChangePasswordError) -> Self {
        match value {
            ChangePasswordError::UserNotFound => {
                Self::UnprocessableEntity("user not found".to_string())
            }
            ChangePasswordError::Database(e) => e.log_500(),
            ChangePasswordError::AuthenticateUserError(e) => ApiError::from(e),
        }
    }
}

#[cfg(feature = "zerver")]
impl From<InvalidChangePassword> for ApiError {
    fn from(value: InvalidChangePassword) -> Self {
        match value {
            InvalidChangePassword::Password(e) => {
                Self::UnprocessableEntity(format!("invalid password {}", e))
            }
            InvalidChangePassword::SameAsCurrent => Self::UnprocessableEntity(
                "new password must be different from your current password".to_string(),
            ),
            InvalidChangePassword::FailedPasswordHash(e) => e.log_500(),
        }
    }
}

/// Changes the user's password after verifying the current one.
#[cfg(feature = "zerver")]
pub async fn change_password(
    user: AuthenticatedUser,
    State(state): State<AppState>,
    Json(body): Json<HttpChangePassword>,
) -> Result<(StatusCode, Json<()>), ApiError> {
    let request = ChangePassword::new(user.id, &body.current_password, &body.new_password)?;

    state
        .auth_service
        .change_password_and_revoke_sessions(&request)
        .await
        .map_err(ApiError::from)?;

    let metrics = std::sync::Arc::clone(&state.metrics_service);
    let uid = user.id;
    tokio::spawn(async move {
        if let Err(e) = metrics
            .record_audit(uid, AuditAction::PasswordChanged)
            .await
        {
            tracing::warn!(error = ?e, "metrics: audit password change failed");
        }
    });

    Ok((StatusCode::OK, Json(())))
}
