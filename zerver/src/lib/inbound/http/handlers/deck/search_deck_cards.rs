#[cfg(feature = "zerver")]
use crate::{
    domain::{
        auth::ports::AuthService,
        card::ports::CardService,
        deck::{
            models::deck::search_deck_cards::SearchDeckCardsError,
            ports::DeckService,
        },
        health::ports::HealthService,
        user::ports::UserService,
    },
    inbound::http::{middleware::AuthenticatedUser, ApiError, AppState},
};
#[cfg(feature = "zerver")]
use zwipe_core::domain::{
    card::{Card, search_card::card_filter::CardFilter},
    deck::requests::get_deck_profile::GetDeckProfile,
};
#[cfg(feature = "zerver")]
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
#[cfg(feature = "zerver")]
use uuid::Uuid;

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

/// Deck-aware card search: same `CardFilter` body as the plain search, but
/// scoped to a deck — cards already in the deck (any board, plus profile
/// slots) are excluded, and results default to synergy ordering when no
/// explicit `order_by` is set.
#[cfg(feature = "zerver")]
pub async fn search_deck_cards<AS, US, HS, CS, DS>(
    user: AuthenticatedUser,
    State(state): State<AppState<AS, US, HS, CS, DS>>,
    Path(deck_id): Path<Uuid>,
    Json(filter): Json<CardFilter>,
) -> Result<(StatusCode, Json<Vec<Card>>), ApiError>
where
    AS: AuthService,
    US: UserService,
    HS: HealthService,
    CS: CardService,
    DS: DeckService,
{
    let request = GetDeckProfile::new(user.id, deck_id);

    state
        .deck_service
        .search_deck_cards(&request, &filter)
        .await
        .map_err(ApiError::from)
        .map(|cards| (StatusCode::OK, Json(cards)))
}
