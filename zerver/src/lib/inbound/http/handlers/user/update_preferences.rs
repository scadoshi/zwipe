//! Update user preferences handler.

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use zwipe_core::domain::user::preferences::{UpdatePreferences, UserPreferences};
use zwipe_core::http::contracts::user::HttpUpdatePreferences;

use crate::domain::user::models::preferences::{InvalidUpdatePreferences, UpdatePreferencesError};
use crate::inbound::http::middleware::AuthenticatedUser;
use crate::inbound::http::{ApiError, AppState, To500};

impl From<InvalidUpdatePreferences> for ApiError {
    fn from(value: InvalidUpdatePreferences) -> Self {
        Self::UnprocessableEntity(value.to_string())
    }
}

impl From<UpdatePreferencesError> for ApiError {
    fn from(value: UpdatePreferencesError) -> Self {
        match value {
            UpdatePreferencesError::Invalid(e) => Self::UnprocessableEntity(e.to_string()),
            UpdatePreferencesError::Database(e) => e.to_500(),
        }
    }
}

/// Updates the authenticated user's display preferences.
pub async fn update_preferences(
    user: AuthenticatedUser,
    State(state): State<AppState>,
    Json(body): Json<HttpUpdatePreferences>,
) -> Result<(StatusCode, Json<UserPreferences>), ApiError> {
    let request = UpdatePreferences::new(
        user.id,
        body.theme.as_deref(),
        body.dark_mode,
        body.exclude_universes_beyond,
        body.universes_beyond_exceptions,
        body.pip_style,
    )
    .map_err(ApiError::from)?;

    state
        .user_service
        .update_preferences(&request)
        .await
        .map(|prefs| (StatusCode::OK, Json(prefs)))
        .map_err(ApiError::from)
}
