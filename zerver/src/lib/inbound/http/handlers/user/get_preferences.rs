//! Get user preferences handler.

#[cfg(feature = "zerver")]
use crate::{
    domain::user::models::preferences::GetPreferencesError,
    inbound::http::{ApiError, AppState, Log500, middleware::AuthenticatedUser},
};
#[cfg(feature = "zerver")]
use axum::{Json, extract::State, http::StatusCode};
#[cfg(feature = "zerver")]
use zwipe_core::domain::user::preferences::UserPreferences;

#[cfg(feature = "zerver")]
impl From<GetPreferencesError> for ApiError {
    fn from(value: GetPreferencesError) -> Self {
        match value {
            GetPreferencesError::Database(e) => e.log_500(),
        }
    }
}

/// Returns the authenticated user's display preferences.
#[cfg(feature = "zerver")]
pub async fn get_preferences(
    user: AuthenticatedUser,
    State(state): State<AppState>,
) -> Result<(StatusCode, Json<UserPreferences>), ApiError> {
    state
        .user_service
        .get_preferences(user.id)
        .await
        .map(|prefs| (StatusCode::OK, Json(prefs)))
        .map_err(ApiError::from)
}
