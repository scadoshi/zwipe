//! Mark a one-time UI hint as shown handler.

#[cfg(feature = "zerver")]
use crate::{
    domain::user::models::hints::MarkHintShownError,
    inbound::http::{ApiError, AppState, Log500, middleware::AuthenticatedUser},
};
#[cfg(feature = "zerver")]
use axum::{Json, extract::State, http::StatusCode};
#[cfg(feature = "zerver")]
use zwipe_core::domain::user::{
    User,
    models::hints::{InvalidHintKey, MarkHintShown},
};
#[cfg(feature = "zerver")]
use zwipe_core::http::contracts::user::HttpMarkHintShown;

#[cfg(feature = "zerver")]
impl From<InvalidHintKey> for ApiError {
    fn from(value: InvalidHintKey) -> Self {
        Self::UnprocessableEntity(value.to_string())
    }
}

#[cfg(feature = "zerver")]
impl From<MarkHintShownError> for ApiError {
    fn from(value: MarkHintShownError) -> Self {
        match value {
            MarkHintShownError::NotFound => Self::NotFound("user not found".to_string()),
            MarkHintShownError::Database(e) | MarkHintShownError::UserFromDb(e) => e.log_500(),
        }
    }
}

/// Marks a one-time UI hint as shown for the authenticated user.
///
/// Idempotent: marking an already-shown hint is a no-op. Responds with the
/// updated user so the client can sync its session in place.
#[cfg(feature = "zerver")]
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
