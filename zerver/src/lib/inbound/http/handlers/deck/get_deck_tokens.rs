//! Get tokens produced by a deck's cards.

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use uuid::Uuid;
use zwipe_core::domain::card::Card;

use crate::domain::deck::models::deck::get_deck_tokens::GetDeckTokensError;
use crate::inbound::http::middleware::AuthenticatedUser;
use crate::inbound::http::{ApiError, AppState};

impl From<GetDeckTokensError> for ApiError {
    fn from(value: GetDeckTokensError) -> Self {
        match value {
            GetDeckTokensError::GetDeckError(e) => ApiError::from(e),
            GetDeckTokensError::GetCardError(e) => ApiError::from(e),
        }
    }
}

/// Returns all token cards produced by the cards in a deck.
///
/// # Errors
///
/// Returns 404 when the deck is missing or isn't the caller's, and 500 on a
/// database failure.
pub async fn get_deck_tokens(
    user: AuthenticatedUser,
    State(state): State<AppState>,
    Path(deck_id): Path<Uuid>,
) -> Result<(StatusCode, Json<Vec<Card>>), ApiError> {
    use zwipe_core::domain::deck::requests::get_deck_profile::GetDeckProfile;

    let request = GetDeckProfile::new(user.id, deck_id);

    state
        .deck_service
        .fetch_deck_tokens(&request)
        .await
        .map_err(ApiError::from)
        .map(|tokens| (StatusCode::OK, Json(tokens)))
}
