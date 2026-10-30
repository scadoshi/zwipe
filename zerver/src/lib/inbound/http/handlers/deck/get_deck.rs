//! Full deck with cards handler.

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use uuid::Uuid;
use zwipe_core::domain::deck::Deck;

use crate::domain::deck::models::deck::get_deck::GetDeckError;
use crate::inbound::http::middleware::AuthenticatedUser;
use crate::inbound::http::{ApiError, AppState};

impl From<GetDeckError> for ApiError {
    fn from(value: GetDeckError) -> Self {
        match value {
            GetDeckError::GetCardError(e) => ApiError::from(e),
            GetDeckError::GetDeckCardError(e) => ApiError::from(e),
            GetDeckError::GetCardProfileError(e) => ApiError::from(e),
            GetDeckError::GetDeckProfileError(e) => ApiError::from(e),
        }
    }
}

/// Returns the full deck including all cards (`get_deck_profile` returns only the metadata).
///
/// # Errors
///
/// Returns 404 when the deck is missing or isn't the caller's, and 500 on a
/// database failure.
pub async fn get_deck(
    user: AuthenticatedUser,
    State(state): State<AppState>,
    Path(deck_id): Path<Uuid>,
) -> Result<(StatusCode, Json<Deck>), ApiError> {
    use zwipe_core::domain::deck::requests::get_deck_profile::GetDeckProfile;

    let request = GetDeckProfile::new(user.id, deck_id);

    state
        .deck_service
        .fetch_deck(&request)
        .await
        .map_err(ApiError::from)
        .map(|deck| (StatusCode::OK, Json(deck)))
}
