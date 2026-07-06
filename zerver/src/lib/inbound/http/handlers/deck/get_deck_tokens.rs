//! Get tokens produced by a deck's cards.

#[cfg(feature = "zerver")]
use crate::{
    domain::deck::models::deck::get_deck_tokens::GetDeckTokensError,
    inbound::http::{ApiError, AppState, middleware::AuthenticatedUser},
};
#[cfg(feature = "zerver")]
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
#[cfg(feature = "zerver")]
use uuid::Uuid;
#[cfg(feature = "zerver")]
use zwipe_core::domain::card::Card;

#[cfg(feature = "zerver")]
impl From<GetDeckTokensError> for ApiError {
    fn from(value: GetDeckTokensError) -> Self {
        match value {
            GetDeckTokensError::GetDeckError(e) => ApiError::from(e),
            GetDeckTokensError::GetCardError(e) => ApiError::from(e),
        }
    }
}

/// Returns all token cards produced by the cards in a deck.
#[cfg(feature = "zerver")]
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
