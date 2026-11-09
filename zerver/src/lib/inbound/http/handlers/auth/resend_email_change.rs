//! Re-send a pending email change's link handler (private).

use axum::extract::State;
use axum::http::StatusCode;
use zwipe_core::domain::user::requests::get_user::GetUser;

use crate::domain::auth::ports::AuthService;
use crate::domain::auth::requests::resend_email_change::ResendEmailChangeError;
use crate::domain::user::ports::UserService;
use crate::inbound::http::middleware::AuthenticatedUser;
use crate::inbound::http::{ApiError, AppState, To500};

impl From<ResendEmailChangeError> for ApiError {
    fn from(value: ResendEmailChangeError) -> Self {
        match value {
            ResendEmailChangeError::NoPendingChange => {
                Self::NotFound("no pending email change".to_string())
            }
            ResendEmailChangeError::Database(e) => e.to_500(),
        }
    }
}

/// Mails a fresh link for the authenticated user's pending email change,
/// replacing the old link and its expiry.
///
/// # Errors
///
/// Returns 404 when there is no pending change or the user is gone, and 500
/// when the lookup or the store fails. The route's limiter answers a second
/// press within a minute with 429.
pub async fn resend_email_change(
    user: AuthenticatedUser,
    State(state): State<AppState>,
) -> Result<StatusCode, ApiError> {
    let profile = state
        .user_service
        .fetch_user(&GetUser::from(user.id))
        .await
        .map_err(ApiError::from)?;

    state
        .auth_service
        .resend_email_change(&profile)
        .await
        .map_err(ApiError::from)
        .map(|()| StatusCode::OK)
}
