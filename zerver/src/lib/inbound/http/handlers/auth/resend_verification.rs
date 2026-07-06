#[cfg(feature = "zerver")]
use axum::{extract::State, http::StatusCode};

#[cfg(feature = "zerver")]
use crate::inbound::http::{ApiError, AppState, middleware::AuthenticatedUser};
#[cfg(feature = "zerver")]
use zwipe_core::domain::user::requests::get_user::GetUser;

/// Re-sends the email verification link to the authenticated user.
///
/// Returns `422` if the email is already verified.
#[cfg(feature = "zerver")]
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
