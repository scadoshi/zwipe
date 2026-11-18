//! Create new deck.

use uuid::Uuid;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::deck::deck_profile::DeckProfile;
use zwipe_core::http::contracts::deck::HttpCreateDeckProfile;
use zwipe_core::http::endpoints::deck::CreateDeck;

use crate::{ClientError, ZwipeClient};

impl ZwipeClient {
    /// Creates a deck under a deck id of its own: the request's, or a fresh one when it has
    /// none, so each call is one create. When the reply is lost ([`ClientError::Network`]), the
    /// deck is looked up by that id: a create that landed answers as created, and one that
    /// didn't answers the network error. A server that ignores the id answers the error too.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError`] when the request fails to complete and the deck isn't there,
    /// the server answers with an error status, or the reply doesn't decode.
    pub async fn create_deck_profile(
        &self,
        request: &HttpCreateDeckProfile,
        session: &Session,
    ) -> Result<DeckProfile, ClientError> {
        let id = request.id.unwrap_or_else(Uuid::new_v4);
        let request = HttpCreateDeckProfile {
            id: Some(id),
            ..request.clone()
        };
        match self.call(CreateDeck(request), Some(session)).await {
            Err(lost @ ClientError::Network(_)) => {
                self.fetch_deck_profile(id, session).await.map_err(|_| lost)
            }
            answered => answered,
        }
    }
}
