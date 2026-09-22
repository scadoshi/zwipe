//! Card search with filters.

use crate::{ClientError, ZwipeClient};
use zwipe_core::{
    domain::{
        auth::models::session::Session,
        card::{Card, search_card::card_filter::CardQuery},
    },
    http::endpoints::card::SearchCards,
};

impl ZwipeClient {
    /// Searches cards with filter criteria.
    pub async fn search_cards(
        &self,
        card_filter: &CardQuery,
        session: &Session,
    ) -> Result<Vec<Card>, ClientError> {
        self.call(SearchCards(card_filter.clone()), Some(session))
            .await
    }
}
