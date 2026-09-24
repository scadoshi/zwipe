use crate::{
    domain::deck::models::deck::search_deck_cards::SearchDeckCardsError,
    inbound::http::{ApiError, AppState, middleware::AuthenticatedUser},
};
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use uuid::Uuid;
use zwipe_core::{
    domain::{
        card::search_card::card_filter::CardQuery, deck::requests::get_deck_profile::GetDeckProfile,
    },
    http::contracts::deck::HttpDeckCardSearch,
};

impl From<SearchDeckCardsError> for ApiError {
    fn from(value: SearchDeckCardsError) -> Self {
        use crate::inbound::http::To500;

        match value {
            SearchDeckCardsError::GetDeckProfile(e) => e.into(),
            SearchDeckCardsError::SearchCards(e) => e.into(),
            SearchDeckCardsError::Database(e) => e.to_500(),
        }
    }
}

/// Deck-aware card search: same `CardQuery` body as the plain search, but
/// scoped to a deck: cards already in the deck (any board, plus profile
/// slots) are excluded, and results default to synergy ordering when no
/// explicit `order_by` is set.
pub async fn search_deck_cards(
    user: AuthenticatedUser,
    State(state): State<AppState>,
    Path(deck_id): Path<Uuid>,
    Json(filter): Json<CardQuery>,
) -> Result<(StatusCode, Json<HttpDeckCardSearch>), ApiError> {
    let request = GetDeckProfile::new(user.id, deck_id);

    state
        .deck_service
        .search_deck_cards(&request, &filter)
        .await
        .map_err(ApiError::from)
        .map(|(cards, synergy_warming)| {
            (
                StatusCode::OK,
                Json(HttpDeckCardSearch {
                    cards,
                    synergy_applied: !synergy_warming,
                }),
            )
        })
}
