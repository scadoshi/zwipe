//! Add a card to a deck.

use uuid::Uuid;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::deck::deck_card::DeckCard;
use zwipe_core::http::contracts::deck_card::HttpCreateDeckCard;
use zwipe_core::http::endpoints::deck::CreateDeckCard;

use crate::{ClientError, ZwipeClient};

impl ZwipeClient {
    /// Adds cards to a deck.
    pub async fn create_deck_card(
        &self,
        deck_id: Uuid,
        request: &HttpCreateDeckCard,
        session: &Session,
    ) -> Result<DeckCard, ClientError> {
        self.call(CreateDeckCard(deck_id, request.clone()), Some(session))
            .await
    }
}
