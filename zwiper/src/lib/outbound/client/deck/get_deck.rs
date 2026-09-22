//! Fetch a deck with all its cards.

use crate::outbound::client::{ClientError, ZwipeClient};
use uuid::Uuid;
use zwipe_core::{
    domain::{auth::models::session::Session, deck::Deck},
    http::endpoints::deck::GetDeck,
};

impl ZwipeClient {
    /// Fetches a complete deck with all cards.
    pub async fn get_deck(&self, deck_id: Uuid, session: &Session) -> Result<Deck, ClientError> {
        self.call(GetDeck(deck_id), Some(session)).await
    }
}
