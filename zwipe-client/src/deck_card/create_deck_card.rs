//! Add a card to a deck.

use crate::{ClientError, ZwipeClient};
use uuid::Uuid;
use zwipe_core::{
    domain::{auth::models::session::Session, deck::deck_card::DeckCard},
    http::{contracts::deck_card::HttpCreateDeckCard, endpoints::deck::CreateDeckCard},
};

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
