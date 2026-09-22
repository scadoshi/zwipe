//! Commander search: the dedicated select-serving path.
//!
//! Same `CardQuery` card_filter.clone() as the plain search, but the server orders by
//! decks-helmed popularity, banded + wildcarded per user per day, with
//! token/emblem printings excluded. An explicit sort in the filter still
//! wins. (context/archive/commander_select_ordering.md)

use crate::{ClientError, ZwipeClient};
use zwipe_core::{
    domain::{
        auth::models::session::Session,
        card::{Card, search_card::card_filter::CardQuery},
    },
    http::endpoints::card::SearchCommanders,
};

impl ZwipeClient {
    /// Searches commander candidates.
    pub async fn search_commanders(
        &self,
        card_filter: &CardQuery,
        session: &Session,
    ) -> Result<Vec<Card>, ClientError> {
        self.call(SearchCommanders(card_filter.clone()), Some(session))
            .await
    }
}
