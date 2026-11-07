//! Update user settings handler.

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use zwipe_core::domain::user::settings::{UpdateSettings, UserSettings};
use zwipe_core::http::contracts::user::HttpUpdateSettings;

use crate::domain::user::models::settings::{InvalidUpdateSettings, UpdateSettingsError};
use crate::domain::user::ports::UserService;
use crate::inbound::http::middleware::AuthenticatedUser;
use crate::inbound::http::{ApiError, AppState, To500};

impl From<InvalidUpdateSettings> for ApiError {
    fn from(value: InvalidUpdateSettings) -> Self {
        Self::UnprocessableEntity(value.to_string())
    }
}

impl From<UpdateSettingsError> for ApiError {
    fn from(value: UpdateSettingsError) -> Self {
        match value {
            UpdateSettingsError::Invalid(e) => Self::UnprocessableEntity(e.to_string()),
            UpdateSettingsError::Database(e) => e.to_500(),
        }
    }
}

/// Updates the authenticated user's display settings.
///
/// # Errors
///
/// Returns 422 for an invalid setting, and 500 on a database failure.
pub async fn update_settings(
    user: AuthenticatedUser,
    State(state): State<AppState>,
    Json(body): Json<HttpUpdateSettings>,
) -> Result<(StatusCode, Json<UserSettings>), ApiError> {
    let request = UpdateSettings::new(
        user.id,
        body.theme.as_deref(),
        body.dark_mode,
        body.exclude_universes_beyond,
        body.universes_beyond_exceptions,
        body.pip_style,
        body.swipe_confirmations,
    )
    .map_err(ApiError::from)?;

    state
        .user_service
        .update_settings(&request)
        .await
        .map(|settings| (StatusCode::OK, Json(settings)))
        .map_err(ApiError::from)
}
