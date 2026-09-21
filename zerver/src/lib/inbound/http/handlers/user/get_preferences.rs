//! Get user preferences handler.

use crate::{
    domain::user::models::preferences::GetPreferencesError,
    inbound::http::{ApiError, AppState, To500, middleware::AuthenticatedUser},
};
use axum::{Json, extract::State, http::StatusCode};
use zwipe_core::domain::user::preferences::UserPreferences;

impl From<GetPreferencesError> for ApiError {
    fn from(value: GetPreferencesError) -> Self {
        match value {
            GetPreferencesError::Database(e) => e.to_500(),
        }
    }
}

/// Returns the authenticated user's display preferences.
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
