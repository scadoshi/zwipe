//! Delete existing deck.

use crate::outbound::client::{ClientError, ZwipeClient};
use std::future::Future;
use uuid::Uuid;
use zwipe_core::{domain::auth::models::session::Session, http::endpoints::deck::DeleteDeck};

/// Trait for deleting decks by ID.
#[allow(missing_docs)]
pub trait ClientDeleteDeck {
    fn delete_deck(
        &self,
        deck_id: Uuid,
        session: &Session,
    ) -> impl Future<Output = Result<(), ClientError>> + Send;
}

impl ClientDeleteDeck for ZwipeClient {
    async fn delete_deck(&self, deck_id: Uuid, session: &Session) -> Result<(), ClientError> {
        self.call(DeleteDeck(deck_id), Some(session)).await
    }
}
