#[cfg(feature = "zerver")]
use axum::{extract::State, http::StatusCode, Json};
#[cfg(feature = "zerver")]
use zwipe_core::http::contracts::auth::HttpChangePassword;

#[cfg(feature = "zerver")]
use crate::{
    domain::{
        auth::{
            requests::change_password::{ChangePassword, ChangePasswordError, InvalidChangePassword},
            ports::AuthService,
        },
        card::ports::CardService,
        deck::ports::DeckService,
        health::ports::HealthService,
        metrics::models::kinds::AuditAction,
        user::ports::UserService,
    },
    inbound::http::{middleware::AuthenticatedUser, ApiError, AppState, Log500},
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
            InvalidChangePassword::FailedPasswordHash(e) => e.log_500(),
        }
    }
}

/// Changes the user's password after verifying the current one.
#[cfg(feature = "zerver")]
pub async fn change_password<AS, US, HS, CS, DS>(
    user: AuthenticatedUser,
    State(state): State<AppState<AS, US, HS, CS, DS>>,
    Json(body): Json<HttpChangePassword>,
) -> Result<(StatusCode, Json<()>), ApiError>
where
    AS: AuthService,
    US: UserService,
    HS: HealthService,
    CS: CardService,
    DS: DeckService,
{
    let request = ChangePassword::new(user.id, &body.current_password, &body.new_password)?;

    state
        .auth_service
        .change_password_and_revoke_sessions(&request)
        .await
        .map_err(ApiError::from)?;

    let metrics = std::sync::Arc::clone(&state.metrics_service);
    let uid = user.id;
    tokio::spawn(async move {
        if let Err(e) = metrics.record_audit(uid, AuditAction::PasswordChanged).await {
            tracing::warn!(error = ?e, "metrics: audit password change failed");
        }
    });

    Ok((StatusCode::OK, Json(())))
}
