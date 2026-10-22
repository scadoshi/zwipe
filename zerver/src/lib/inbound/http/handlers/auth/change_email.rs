use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use zwipe_core::domain::user::User;
use zwipe_core::http::contracts::auth::HttpChangeEmail;

use crate::domain::auth::requests::change_email::{
    ChangeEmail, ChangeEmailError, ChangeEmailOutcome, InvalidChangeEmail,
};
use crate::domain::metrics::models::kinds::AuditAction;
use crate::inbound::http::middleware::AuthenticatedUser;
use crate::inbound::http::{ApiError, AppState, To500};

impl From<ChangeEmailError> for ApiError {
    fn from(value: ChangeEmailError) -> Self {
        match value {
            ChangeEmailError::UserNotFound => Self::NotFound("user not found".to_string()),
            ChangeEmailError::Database(e) => e.to_500(),
            ChangeEmailError::UserFromDb(e) => e.to_500(),
            ChangeEmailError::AuthenticateUserError(e) => ApiError::from(e),
            // The service answers a taken address with `ChangeEmailOutcome::Taken`;
            // a `Duplicate` reaching here is a bug, and a 500 says nothing about
            // the address.
            ChangeEmailError::Duplicate => value.to_500(),
        }
    }
}

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
///
/// A new address that belongs to another account gets the same 200 and the
/// same user JSON (with the caller's unchanged email) as a real change, so the
/// response never says whether an address is registered. Each arm sends one
/// more email and writes one audit row, so timing does not say either.
pub async fn change_email(
    user: AuthenticatedUser,
    State(state): State<AppState>,
    Json(body): Json<HttpChangeEmail>,
) -> Result<(StatusCode, Json<User>), ApiError> {
    let request = ChangeEmail::new(user.id, &body.email, body.password.read())?;

    let outcome = state
        .auth_service
        .change_email(&request)
        .await
        .map_err(ApiError::from)?;

    let (user, action) = match outcome {
        ChangeEmailOutcome::Changed(user) => {
            if let Err(e) = state
                .auth_service
                .send_verification_email(user.id, user.email.as_ref())
                .await
            {
                tracing::error!(event = "verification_email_failed", user_id = %user.id, error = %e);
            }
            (user, AuditAction::EmailChanged)
        }
        ChangeEmailOutcome::Taken(user) => {
            tracing::info!(event = "email_claim_attempted", user_id = %user.id);
            (user, AuditAction::EmailClaimAttempted)
        }
    };

    let metrics = std::sync::Arc::clone(&state.metrics_service);
    let uid = user.id;
    tokio::spawn(async move {
        if let Err(e) = metrics.record_audit(uid, action).await {
            tracing::warn!(error = ?e, "metrics: audit email change failed");
        }
    });

    Ok((StatusCode::OK, Json(user)))
}
