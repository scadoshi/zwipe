//! Get user settings handler.

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use zwipe_core::domain::user::settings::UserSettings;

use crate::domain::user::models::settings::FetchSettingsError;
use crate::domain::user::ports::UserService;
use crate::inbound::http::middleware::AuthenticatedUser;
use crate::inbound::http::{ApiError, AppState, To500};

impl From<FetchSettingsError> for ApiError {
    fn from(value: FetchSettingsError) -> Self {
        match value {
            FetchSettingsError::Database(e) => e.to_500(),
        }
    }
}

/// Returns the authenticated user's display settings.
///
/// # Errors
///
/// Returns 500 on a database failure.
pub async fn fetch_settings(
    user: AuthenticatedUser,
    State(state): State<AppState>,
) -> Result<(StatusCode, Json<UserSettings>), ApiError> {
    state
        .user_service
        .fetch_settings(user.id)
        .await
        .map(|settings| (StatusCode::OK, Json(settings)))
        .map_err(ApiError::from)
}
