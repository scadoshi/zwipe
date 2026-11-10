//! Cancel a pending email change handler (private).

use axum::extract::State;
use axum::http::StatusCode;

use crate::domain::auth::ports::AuthService;
use crate::domain::auth::requests::cancel_email_change::CancelEmailChangeError;
use crate::inbound::http::middleware::AuthenticatedUser;
use crate::inbound::http::{ApiError, AppState, To500};

impl From<CancelEmailChangeError> for ApiError {
    fn from(value: CancelEmailChangeError) -> Self {
        match value {
            CancelEmailChangeError::NoPendingChange => {
                Self::NotFound("no pending email change".to_string())
            }
            CancelEmailChangeError::Database(e) => e.to_500(),
        }
    }
}

/// Drops the authenticated user's pending email change, so its link no
/// longer confirms and the account keeps its current address.
///
/// # Errors
///
/// Returns 404 when there is no unexpired change pending, and 500 when the
/// delete fails.
pub async fn cancel_email_change(
    user: AuthenticatedUser,
    State(state): State<AppState>,
) -> Result<StatusCode, ApiError> {
    state
        .auth_service
        .cancel_email_change(user.id)
        .await
        .map_err(ApiError::from)
        .map(|()| StatusCode::OK)
}
