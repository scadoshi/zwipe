//! Fetch a deck with all its cards.

use uuid::Uuid;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::deck::Deck;
use zwipe_core::http::endpoints::deck::GetDeck;

use crate::{ClientError, ZwipeClient};

impl ZwipeClient {
    /// Fetches a complete deck with all cards.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError`] when the request fails to complete, the server answers with an
    /// error status, or the reply doesn't decode.
    pub async fn get_deck(&self, deck_id: Uuid, session: &Session) -> Result<Deck, ClientError> {
        self.call(GetDeck(deck_id), Some(session)).await
    }
}
