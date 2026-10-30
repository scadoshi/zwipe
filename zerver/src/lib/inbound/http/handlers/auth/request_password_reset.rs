//! Password reset initiation handler.

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use zwipe_core::http::contracts::auth::HttpRequestPasswordReset;

use crate::domain::auth::requests::request_password_reset::{
    RequestPasswordReset, RequestPasswordResetError,
};
use crate::inbound::http::{ApiError, AppState, To500};

impl From<RequestPasswordResetError> for ApiError {
    fn from(value: RequestPasswordResetError) -> Self {
        match value {
            RequestPasswordResetError::Database(e) => e.to_500(),
        }
    }
}

/// Initiates the password reset flow for the given email address.
///
/// Always returns `200 OK` regardless of whether the email is registered,
/// to prevent email enumeration attacks.
///
/// # Errors
///
/// Returns 500 on a database failure, never anything that depends on the
/// address.
pub async fn request_password_reset(
    State(state): State<AppState>,
    Json(body): Json<HttpRequestPasswordReset>,
) -> Result<StatusCode, ApiError> {
    let request = RequestPasswordReset { email: body.email };
    state
        .auth_service
        .request_password_reset(&request)
        .await
        .map_err(ApiError::from)
        .map(|()| StatusCode::OK)
}
