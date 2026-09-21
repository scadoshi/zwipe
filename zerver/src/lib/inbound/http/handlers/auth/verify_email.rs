use axum::{Json, extract::State, http::StatusCode};
use zwipe_core::http::contracts::auth::HttpVerifyEmail;

use crate::{
    domain::auth::requests::verify_email::{VerifyEmail, VerifyEmailError},
    inbound::http::{ApiError, AppState, To500},
};

impl From<VerifyEmailError> for ApiError {
    fn from(value: VerifyEmailError) -> Self {
        match value {
            VerifyEmailError::InvalidToken => {
                Self::UnprocessableEntity("token not found or expired".to_string())
            }
            VerifyEmailError::Database(e) => e.to_500(),
        }
    }
}

/// Verifies a user's email address using a one-time token.
pub async fn verify_email(
    State(state): State<AppState>,
    Json(body): Json<HttpVerifyEmail>,
) -> Result<StatusCode, ApiError> {
    let request = VerifyEmail { token: body.token };
    state
        .auth_service
        .verify_email(&request)
        .await
        .map_err(ApiError::from)
        .map(|_| StatusCode::OK)
}
