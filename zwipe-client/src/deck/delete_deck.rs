//! Delete existing deck.

use crate::{ClientError, ZwipeClient};
use uuid::Uuid;
use zwipe_core::{domain::auth::models::session::Session, http::endpoints::deck::DeleteDeck};

impl ZwipeClient {
    /// Deletes decks by ID.
    pub async fn delete_deck(&self, deck_id: Uuid, session: &Session) -> Result<(), ClientError> {
        self.call(DeleteDeck(deck_id), Some(session)).await
    }
}
