//! Fetch a deck with all its cards.

use crate::outbound::client::{ClientError, ZwipeClient};
use std::future::Future;
use uuid::Uuid;
use zwipe_core::{
    domain::{auth::models::session::Session, deck::Deck},
    http::endpoints::deck::GetDeck,
};

/// Trait for fetching a complete deck with all cards.
#[allow(missing_docs)]
pub trait ClientGetDeck {
    fn get_deck(
        &self,
        deck_id: Uuid,
        session: &Session,
    ) -> impl Future<Output = Result<Deck, ClientError>> + Send;
}

impl ClientGetDeck for ZwipeClient {
    async fn get_deck(&self, deck_id: Uuid, session: &Session) -> Result<Deck, ClientError> {
        self.call(GetDeck(deck_id), Some(session)).await
    }
}
