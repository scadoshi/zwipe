//! Fetch tokens produced by a deck's cards.

use uuid::Uuid;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::card::Card;
use zwipe_core::http::endpoints::deck::GetDeckTokens;

use crate::{ClientError, ZwipeClient};

impl ZwipeClient {
    /// Fetches all token cards produced by a deck.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError`] when the request fails to complete, the server answers with an
    /// error status, or the reply doesn't decode.
    pub async fn get_deck_tokens(
        &self,
        deck_id: Uuid,
        session: &Session,
    ) -> Result<Vec<Card>, ClientError> {
        self.call(GetDeckTokens(deck_id), Some(session)).await
    }
}
