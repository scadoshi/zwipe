//! Get user preferences handler.

#[cfg(feature = "zerver")]
use crate::{
    domain::{
        auth::ports::AuthService,
        card::ports::CardService,
        deck::ports::DeckService,
        health::ports::HealthService,
        user::{
            models::preferences::{GetPreferencesError, UserPreferences},
            ports::UserService,
        },
    },
    inbound::http::{middleware::AuthenticatedUser, ApiError, AppState, Log500},
};
#[cfg(feature = "zerver")]
use axum::{Json, extract::State, http::StatusCode};

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
pub async fn get_preferences<AS, US, HS, CS, DS>(
    user: AuthenticatedUser,
    State(state): State<AppState<AS, US, HS, CS, DS>>,
) -> Result<(StatusCode, Json<UserPreferences>), ApiError>
where
    AS: AuthService,
    US: UserService,
    HS: HealthService,
    CS: CardService,
    DS: DeckService,
{
    state
        .user_service
        .get_preferences(user.id)
        .await
        .map(|prefs| (StatusCode::OK, Json(prefs)))
        .map_err(ApiError::from)
}
