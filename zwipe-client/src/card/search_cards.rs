//! Card search with filters.

use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::search_card::card_filter::CardQuery;
use zwipe_core::http::endpoints::card::SearchCards;

use crate::{ClientError, ZwipeClient};

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
