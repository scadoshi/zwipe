#[cfg(feature = "zerver")]
use crate::{
    domain::deck::models::deck::search_deck_cards::SearchDeckCardsError,
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
use zwipe_core::domain::{
    card::{Card, search_card::card_filter::CardQuery},
    deck::requests::get_deck_profile::GetDeckProfile,
};

#[cfg(feature = "zerver")]
impl From<SearchDeckCardsError> for ApiError {
    fn from(value: SearchDeckCardsError) -> Self {
        use crate::inbound::http::Log500;

        match value {
            SearchDeckCardsError::GetDeckProfile(e) => e.into(),
            SearchDeckCardsError::SearchCards(e) => e.into(),
            SearchDeckCardsError::Database(e) => e.log_500(),
        }
    }
}

/// Deck-aware card search: same `CardQuery` body as the plain search, but
/// scoped to a deck — cards already in the deck (any board, plus profile
/// slots) are excluded, and results default to synergy ordering when no
/// explicit `order_by` is set.
#[cfg(feature = "zerver")]
pub async fn search_deck_cards(
    user: AuthenticatedUser,
    State(state): State<AppState>,
    Path(deck_id): Path<Uuid>,
    Json(filter): Json<CardQuery>,
) -> Result<
    (
        StatusCode,
        [(&'static str, &'static str); 1],
        Json<Vec<Card>>,
    ),
    ApiError,
> {
    let request = GetDeckProfile::new(user.id, deck_id);

    state
        .deck_service
        .search_deck_cards(&request, &filter)
        .await
        .map_err(ApiError::from)
        .map(|(cards, synergy_warming)| {
            // Signal cold-synergy fallback via a header — the body stays a bare
            // card array, so older clients (which ignore the header) keep working.
            let applied = if synergy_warming { "false" } else { "true" };
            (
                StatusCode::OK,
                [("x-synergy-applied", applied)],
                Json(cards),
            )
        })
}
