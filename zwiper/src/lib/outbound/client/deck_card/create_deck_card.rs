//! Add a card to a deck.

use crate::outbound::client::{ClientError, ZwipeClient};
use std::future::Future;
use uuid::Uuid;
use zwipe_core::{
    domain::{auth::models::session::Session, deck::deck_card::DeckCard},
    http::{contracts::deck_card::HttpCreateDeckCard, endpoints::deck::CreateDeckCard},
};

/// Trait for adding cards to a deck.
#[allow(missing_docs)]
pub trait ClientCreateDeckCard {
    fn create_deck_card(
        &self,
        deck_id: Uuid,
        request: &HttpCreateDeckCard,
        session: &Session,
    ) -> impl Future<Output = Result<DeckCard, ClientError>> + Send;
}

impl ClientCreateDeckCard for ZwipeClient {
    async fn create_deck_card(
        &self,
        deck_id: Uuid,
        request: &HttpCreateDeckCard,
        session: &Session,
    ) -> Result<DeckCard, ClientError> {
        let body = serde_json::to_value(request)?;
        self.call(CreateDeckCard(deck_id, body), Some(session))
            .await
    }
}
