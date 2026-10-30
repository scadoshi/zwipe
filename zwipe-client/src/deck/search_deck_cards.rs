//! Deck-aware card search.
//!
//! Same `CardQuery` body as the plain search, but scoped to a deck: the
//! server excludes cards already in the deck (any board, plus profile slots)
//! and defaults to synergy ordering when no explicit sort is set.

use uuid::Uuid;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::card::search_card::card_filter::CardQuery;
use zwipe_core::http::contracts::deck::HttpDeckCardSearch;
use zwipe_core::http::endpoints::deck::SearchDeckCards;

use crate::{ClientError, ZwipeClient};

impl ZwipeClient {
    /// Deck-aware card search: the page of cards, whether synergy applied
    /// (false while the commander's cache is still warming, so the server
    /// served the full pool), and where related cards begin when the synergy
    /// pool ran out.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError`] when the request fails to complete, the server answers with an
    /// error status, or the reply doesn't decode.
    pub async fn search_deck_cards(
        &self,
        deck_id: Uuid,
        card_filter: &CardQuery,
        session: &Session,
    ) -> Result<HttpDeckCardSearch, ClientError> {
        self.call(SearchDeckCards(deck_id, card_filter.clone()), Some(session))
            .await
    }
}
