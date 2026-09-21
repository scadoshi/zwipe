//! Get tokens produced by a deck's cards.

use crate::{
    domain::deck::models::deck::get_deck_tokens::GetDeckTokensError,
    inbound::http::{ApiError, AppState, middleware::AuthenticatedUser},
};
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use uuid::Uuid;
use zwipe_core::domain::card::Card;

impl From<GetDeckTokensError> for ApiError {
    fn from(value: GetDeckTokensError) -> Self {
        match value {
            GetDeckTokensError::GetDeckError(e) => ApiError::from(e),
            GetDeckTokensError::GetCardError(e) => ApiError::from(e),
        }
    }
}

/// Returns all token cards produced by the cards in a deck.
pub async fn get_deck_tokens(
    user: AuthenticatedUser,
    State(state): State<AppState>,
    Path(deck_id): Path<Uuid>,
) -> Result<(StatusCode, Json<Vec<Card>>), ApiError> {
    use zwipe_core::domain::deck::requests::get_deck_profile::GetDeckProfile;

    let request = GetDeckProfile::new(user.id, deck_id);

    state
        .deck_service
        .get_deck_tokens(&request)
        .await
        .map_err(ApiError::from)
        .map(|tokens| (StatusCode::OK, Json(tokens)))
}
