//! Deck-aware card search.
//!
//! Same `CardQuery` body as the plain search, but scoped to a deck: the
//! server excludes cards already in the deck (any board, plus profile slots)
//! and defaults to synergy ordering when no explicit sort is set.

use crate::{ClientError, ZwipeClient};
use uuid::Uuid;
use zwipe_core::{
    domain::{
        auth::models::session::Session,
        card::{Card, search_card::card_filter::CardQuery},
    },
    http::endpoints::deck::SearchDeckCards,
};

impl ZwipeClient {
    /// Deck-aware card search.
    ///
    /// Returns `(cards, synergy_warming)`: `synergy_warming` is true when
    /// synergy was requested but the commander's cache was still warming, so
    /// the server served the full pool instead of a synergy ordering.
    pub async fn search_deck_cards(
        &self,
        deck_id: Uuid,
        card_filter: &CardQuery,
        session: &Session,
    ) -> Result<(Vec<Card>, bool), ClientError> {
        let body = self
            .call(SearchDeckCards(deck_id, card_filter.clone()), Some(session))
            .await?;
        Ok((body.cards, !body.synergy_applied))
    }
}
