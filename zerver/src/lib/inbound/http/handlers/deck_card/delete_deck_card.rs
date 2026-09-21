use axum::{
    extract::{Path, State},
    http::StatusCode,
};

use crate::{
    domain::deck::models::deck_card::delete_deck_card::DeleteDeckCardError,
    inbound::http::{
        ApiError, AppState, To500, handlers::metrics::check_completion::check_deck_completion,
        middleware::AuthenticatedUser,
    },
};
use zwipe_core::domain::deck::requests::delete_deck_card::{DeleteDeckCard, InvalidDeleteDeckCard};

impl From<DeleteDeckCardError> for ApiError {
    fn from(value: DeleteDeckCardError) -> Self {
        match value {
            DeleteDeckCardError::NotFound => {
                Self::UnprocessableEntity("deck card not found".to_string())
            }
            DeleteDeckCardError::Database(e) => e.to_500(),
            DeleteDeckCardError::GetDeckProfileError(e) => ApiError::from(e),
            DeleteDeckCardError::Forbidden => {
                Self::Forbidden(DeleteDeckCardError::Forbidden.to_string())
            }
        }
    }
}

impl From<InvalidDeleteDeckCard> for ApiError {
    fn from(value: InvalidDeleteDeckCard) -> Self {
        match value {
            InvalidDeleteDeckCard::DeckId(e) => {
                Self::UnprocessableEntity(format!("invalid deck id: {}", e))
            }
            InvalidDeleteDeckCard::ScryfallDataId(e) => {
                Self::UnprocessableEntity(format!("invalid card profile id: {}", e))
            }
        }
    }
}

/// Removes a card from a deck.
pub async fn delete_deck_card(
    user: AuthenticatedUser,
    State(state): State<AppState>,
    Path((deck_id, scryfall_data_id)): Path<(String, String)>,
) -> Result<StatusCode, ApiError> {
    let request = DeleteDeckCard::new(user.id, &deck_id, &scryfall_data_id)?;

    state
        .deck_service
        .delete_deck_card(&request)
        .await
        .map_err(ApiError::from)?;

    let metrics = std::sync::Arc::clone(&state.metrics_service);
    let deck_service = std::sync::Arc::clone(&state.deck_service);
    let uid = user.id;
    let did = request.deck_id;
    tokio::spawn(check_deck_completion(deck_service, metrics, uid, did));

    Ok(StatusCode::NO_CONTENT)
}
