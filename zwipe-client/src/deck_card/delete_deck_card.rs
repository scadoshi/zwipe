//! Remove a card from a deck.

use uuid::Uuid;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::http::endpoints::deck::DeleteDeckCard;

use crate::{ClientError, ZwipeClient};

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
