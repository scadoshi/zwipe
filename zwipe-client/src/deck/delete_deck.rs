//! Delete existing deck.

use uuid::Uuid;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::http::endpoints::deck::DeleteDeck;

use crate::{ClientError, ZwipeClient};

impl ZwipeClient {
    /// Deletes decks by ID.
    pub async fn delete_deck(&self, deck_id: Uuid, session: &Session) -> Result<(), ClientError> {
        self.call(DeleteDeck(deck_id), Some(session)).await
    }
}
