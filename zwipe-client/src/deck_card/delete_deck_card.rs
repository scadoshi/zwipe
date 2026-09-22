//! Remove a card from a deck.

use crate::{ClientError, ZwipeClient};
use uuid::Uuid;
use zwipe_core::{domain::auth::models::session::Session, http::endpoints::deck::DeleteDeckCard};

impl ZwipeClient {
    /// Removes cards from a deck.
    pub async fn delete_deck_card(
        &self,
        deck_id: Uuid,
        scryfall_data_id: Uuid,
        session: &Session,
    ) -> Result<(), ClientError> {
        self.call(DeleteDeckCard(deck_id, scryfall_data_id), Some(session))
            .await
    }
}
