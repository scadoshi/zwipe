//! Mark a one-time UI hint as shown handler.

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use zwipe_core::domain::user::User;
use zwipe_core::domain::user::models::hints::{InvalidHintKey, MarkHintShown};
use zwipe_core::http::contracts::user::HttpMarkHintShown;

use crate::domain::user::models::hints::MarkHintShownError;
use crate::inbound::http::middleware::AuthenticatedUser;
use crate::inbound::http::{ApiError, AppState, To500};

impl From<InvalidHintKey> for ApiError {
    fn from(value: InvalidHintKey) -> Self {
        Self::UnprocessableEntity(value.to_string())
    }
}

impl From<MarkHintShownError> for ApiError {
    fn from(value: MarkHintShownError) -> Self {
        match value {
            MarkHintShownError::NotFound => Self::NotFound("user not found".to_string()),
            MarkHintShownError::Database(e) | MarkHintShownError::UserFromDb(e) => e.to_500(),
        }
    }
}

/// Marks a one-time UI hint as shown for the authenticated user.
///
/// Idempotent: marking an already-shown hint is a no-op. Responds with the
/// updated user so the client can sync its session in place.
///
/// # Errors
///
/// Returns 422 for an unknown hint key, 404 when the user is gone, and 500 on a
/// database failure.
pub async fn mark_hint_shown(
    user: AuthenticatedUser,
    State(state): State<AppState>,
    Json(body): Json<HttpMarkHintShown>,
) -> Result<(StatusCode, Json<User>), ApiError> {
    let request = MarkHintShown::new(user.id, &body.hint).map_err(ApiError::from)?;

    state
        .user_service
        .mark_hint_shown(&request)
        .await
        .map(|user| (StatusCode::OK, Json(user)))
        .map_err(ApiError::from)
}
