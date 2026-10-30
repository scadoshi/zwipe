//! Commander search: the dedicated select-serving path.
//!
//! Same `CardQuery` card_filter.clone() as the plain search, but the server orders by
//! decks-helmed popularity, banded + wildcarded per user per day, with
//! token/emblem printings excluded. An explicit sort in the filter still
//! wins. (context/archive/commander_select_ordering.md)

use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::search_card::card_filter::CardQuery;
use zwipe_core::http::endpoints::card::SearchCommanders;

use crate::{ClientError, ZwipeClient};

impl ZwipeClient {
    /// Searches commander candidates.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError`] when the request fails to complete, the server answers with an
    /// error status, or the reply doesn't decode.
    pub async fn search_commanders(
        &self,
        card_filter: &CardQuery,
        session: &Session,
    ) -> Result<Vec<Card>, ClientError> {
        self.call(SearchCommanders(card_filter.clone()), Some(session))
            .await
    }
}
