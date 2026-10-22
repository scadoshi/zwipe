use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use zwipe_core::domain::deck::deck_profile::DeckProfile;
use zwipe_core::domain::deck::requests::get_deck_profiles::{
    GetDeckProfiles, InvalidGetDeckProfiles,
};

use crate::inbound::http::middleware::AuthenticatedUser;
use crate::inbound::http::{ApiError, AppState};

impl From<InvalidGetDeckProfiles> for ApiError {
    fn from(value: InvalidGetDeckProfiles) -> Self {
        match value {
            InvalidGetDeckProfiles::UserId(e) => {
                Self::UnprocessableEntity(format!("invalid user id: {e}"))
            }
        }
    }
}

/// Returns all deck profiles for the authenticated user.
pub async fn get_deck_profiles(
    user: AuthenticatedUser,
    State(state): State<AppState>,
) -> Result<(StatusCode, Json<Vec<DeckProfile>>), ApiError> {
    let request = GetDeckProfiles::new(user.id);
    state
        .deck_service
        .get_deck_profiles(&request)
        .await
        .map_err(ApiError::from)
        .map(|deck_profiles| (StatusCode::OK, Json(deck_profiles)))
}
