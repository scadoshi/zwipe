//! Email change handler.

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use zwipe_core::domain::user::User;
use zwipe_core::http::contracts::auth::HttpChangeEmail;

use crate::domain::auth::requests::change_email::{
    ChangeEmail, ChangeEmailError, InvalidChangeEmail,
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

/// Holds a new email as pending after verifying the password.
///
/// The response is the same whether or not another account has the address;
/// the switch happens when the link sent to it is clicked.
pub async fn change_email(
    user: AuthenticatedUser,
    State(state): State<AppState>,
    Json(body): Json<HttpChangeEmail>,
) -> Result<(StatusCode, Json<User>), ApiError> {
    let request = ChangeEmail::new(user.id, &body.email, body.password.read())?;

    let user = state
        .auth_service
        .request_email_change(&request)
        .await
        .map_err(ApiError::from)?;

    let metrics = std::sync::Arc::clone(&state.metrics_service);
    let uid = user.id;
    tokio::spawn(async move {
        if let Err(e) = metrics
            .record_audit(uid, AuditAction::EmailChangeRequested)
            .await
        {
            tracing::warn!(error = ?e, "metrics: audit email change request failed");
        }
    });

    Ok((StatusCode::OK, Json(user)))
}
