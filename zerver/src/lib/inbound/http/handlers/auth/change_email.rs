#[cfg(feature = "zerver")]
use axum::{extract::State, http::StatusCode, Json};
#[cfg(feature = "zerver")]
use zwipe_core::http::contracts::auth::HttpChangeEmail;

#[cfg(feature = "zerver")]
use crate::{
    domain::{
        auth::{
            requests::change_email::{ChangeEmail, ChangeEmailError, InvalidChangeEmail},
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
use zwipe_core::domain::user::User;

#[cfg(feature = "zerver")]
impl From<ChangeEmailError> for ApiError {
    fn from(value: ChangeEmailError) -> Self {
        match value {
            ChangeEmailError::UserNotFound => Self::NotFound("user not found".to_string()),
            ChangeEmailError::Database(e) => e.log_500(),
            ChangeEmailError::UserFromDb(e) => e.log_500(),
            ChangeEmailError::AuthenticateUserError(e) => ApiError::from(e),
            ChangeEmailError::Duplicate => {
                Self::UnprocessableEntity("email already in use".to_string())
            }
        }
    }
}

#[cfg(feature = "zerver")]
impl From<InvalidChangeEmail> for ApiError {
    fn from(value: InvalidChangeEmail) -> Self {
        match value {
            InvalidChangeEmail::Id(e) => Self::UnprocessableEntity(format!("invalid id: {}", e)),
            InvalidChangeEmail::Email(e) => {
                Self::UnprocessableEntity(format!("invalid email: {}", e))
            }
        }
    }
}

/// Changes the user's email after verifying the password.
#[cfg(feature = "zerver")]
pub async fn change_email<AS, US, HS, CS, DS>(
    user: AuthenticatedUser,
    State(state): State<AppState<AS, US, HS, CS, DS>>,
    Json(body): Json<HttpChangeEmail>,
) -> Result<(StatusCode, Json<User>), ApiError>
where
    AS: AuthService,
    US: UserService,
    HS: HealthService,
    CS: CardService,
    DS: DeckService,
{
    let request = ChangeEmail::new(user.id, &body.email, &body.password)?;

    let updated_user = state
        .auth_service
        .change_email(&request)
        .await
        .map_err(ApiError::from)?;

    if let Err(e) = state
        .auth_service
        .send_verification_email(updated_user.id, updated_user.email.as_ref())
        .await
    {
        tracing::error!(event = "verification_email_failed", user_id = %updated_user.id, error = %e);
    }

    let metrics = std::sync::Arc::clone(&state.metrics_service);
    let uid = updated_user.id;
    tokio::spawn(async move {
        if let Err(e) = metrics.record_audit(uid, AuditAction::EmailChanged).await {
            tracing::warn!(error = ?e, "metrics: audit email change failed");
        }
    });

    Ok((StatusCode::OK, Json(updated_user)))
}
