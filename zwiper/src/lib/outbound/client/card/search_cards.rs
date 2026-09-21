//! Card search with filters.

use crate::outbound::client::{ClientError, ZwipeClient};
use std::future::Future;
use zwipe_core::{
    domain::{
        auth::models::session::Session,
        card::{Card, search_card::card_filter::CardQuery},
    },
    http::endpoints::card::SearchCards,
};

/// Trait for searching cards with filter criteria.
#[allow(missing_docs)]
pub trait ClientSearchCards {
    fn search_cards(
        &self,
        card_filter: &CardQuery,
        session: &Session,
    ) -> impl Future<Output = Result<Vec<Card>, ClientError>> + Send;
}

impl ClientSearchCards for ZwipeClient {
    async fn search_cards(
        &self,
        card_filter: &CardQuery,
        session: &Session,
    ) -> Result<Vec<Card>, ClientError> {
        let body = serde_json::to_value(card_filter)?;
        self.call(SearchCards(body), Some(session)).await
    }
}
