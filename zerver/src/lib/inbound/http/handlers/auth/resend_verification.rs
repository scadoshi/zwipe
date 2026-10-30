//! Re-send email verification handler (private).

use axum::extract::State;
use axum::http::StatusCode;
use zwipe_core::domain::user::requests::get_user::GetUser;

use crate::inbound::http::middleware::AuthenticatedUser;
use crate::inbound::http::{ApiError, AppState};

/// Re-sends the email verification link to the authenticated user.
///
/// Returns `422` if the email is already verified.
pub async fn resend_verification(
    user: AuthenticatedUser,
    State(state): State<AppState>,
) -> Result<StatusCode, ApiError> {
    let profile = state
        .user_service
        .get_user(&GetUser::from(user.id))
        .await
        .map_err(ApiError::from)?;

    if profile.email_verified_at.is_some() {
        return Err(ApiError::UnprocessableEntity(
            "email already verified".to_string(),
        ));
    }

    state
        .auth_service
        .send_verification_email(user.id, profile.email.as_ref())
        .await
        .map_err(ApiError::from)
        .map(|_| StatusCode::OK)
}
