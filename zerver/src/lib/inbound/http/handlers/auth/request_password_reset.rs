use axum::{Json, extract::State, http::StatusCode};
use zwipe_core::http::contracts::auth::HttpRequestPasswordReset;

use crate::{
    domain::auth::requests::request_password_reset::{
        RequestPasswordReset, RequestPasswordResetError,
    },
    inbound::http::{ApiError, AppState, To500},
};

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
        .map(|_| StatusCode::OK)
}
