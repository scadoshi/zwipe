//! Update user preferences handler.

#[cfg(feature = "zerver")]
use crate::{
    domain::{
        auth::ports::AuthService,
        card::ports::CardService,
        deck::ports::DeckService,
        health::ports::HealthService,
        user::{
            models::preferences::{InvalidUpdatePreferences, UpdatePreferencesError},
            ports::UserService,
        },
    },
    inbound::http::{middleware::AuthenticatedUser, ApiError, AppState, Log500},
};
#[cfg(feature = "zerver")]
use axum::{Json, extract::State, http::StatusCode};
pub use zwipe_core::http::contracts::user::HttpUpdatePreferences;
#[cfg(feature = "zerver")]
use zwipe_core::domain::user::preferences::{UpdatePreferences, UserPreferences};

#[cfg(feature = "zerver")]
impl From<InvalidUpdatePreferences> for ApiError {
    fn from(value: InvalidUpdatePreferences) -> Self {
        Self::UnprocessableEntity(value.to_string())
    }
}

#[cfg(feature = "zerver")]
impl From<UpdatePreferencesError> for ApiError {
    fn from(value: UpdatePreferencesError) -> Self {
        match value {
            UpdatePreferencesError::Invalid(e) => Self::UnprocessableEntity(e.to_string()),
            UpdatePreferencesError::Database(e) => e.log_500(),
        }
    }
}

/// Updates the authenticated user's display preferences.
#[cfg(feature = "zerver")]
pub async fn update_preferences<AS, US, HS, CS, DS>(
    user: AuthenticatedUser,
    State(state): State<AppState<AS, US, HS, CS, DS>>,
    Json(body): Json<HttpUpdatePreferences>,
) -> Result<(StatusCode, Json<UserPreferences>), ApiError>
where
    AS: AuthService,
    US: UserService,
    HS: HealthService,
    CS: CardService,
    DS: DeckService,
{
    let request =
        UpdatePreferences::new(user.id, body.theme.as_deref(), body.dark_mode)
            .map_err(ApiError::from)?;

    state
        .user_service
        .update_preferences(&request)
        .await
        .map(|prefs| (StatusCode::OK, Json(prefs)))
        .map_err(ApiError::from)
}
