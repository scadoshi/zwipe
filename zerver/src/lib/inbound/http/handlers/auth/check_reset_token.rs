//! Reset link check handler.

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use zwipe_core::http::contracts::auth::HttpCheckResetToken;

use crate::domain::auth::ports::AuthService;
use crate::domain::auth::requests::check_reset_token::CheckResetTokenError;
use crate::inbound::http::{ApiError, AppState, To500};

impl From<CheckResetTokenError> for ApiError {
    fn from(value: CheckResetTokenError) -> Self {
        match value {
            CheckResetTokenError::InvalidToken => {
                Self::BadRequest("token not found or expired".to_string())
            }
            CheckResetTokenError::Database(e) => e.to_500(),
        }
    }
}

/// Answers whether a reset link's token is still live, so the reset page can
/// say a dead link is dead before a password is chosen. The token is read
/// only: it is neither spent nor extended.
///
/// # Errors
///
/// Returns 400 for an unknown, used or expired token, and 500 on a database
/// failure.
pub async fn check_reset_token(
    State(state): State<AppState>,
    Json(body): Json<HttpCheckResetToken>,
) -> Result<StatusCode, ApiError> {
    state
        .auth_service
        .check_reset_token(&body.token)
        .await
        .map_err(ApiError::from)?;
    Ok(StatusCode::NO_CONTENT)
}
