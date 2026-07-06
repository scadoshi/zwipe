#[cfg(feature = "zerver")]
use crate::inbound::http::{ApiError, AppState, middleware::AuthenticatedUser};
#[cfg(feature = "zerver")]
use axum::{Json, extract::State, http::StatusCode};
#[cfg(feature = "zerver")]
use zwipe_core::domain::deck::{
    deck_profile::DeckProfile,
    requests::get_deck_profiles::{GetDeckProfiles, InvalidGetDeckProfiles},
};

#[cfg(feature = "zerver")]
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
#[cfg(feature = "zerver")]
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
