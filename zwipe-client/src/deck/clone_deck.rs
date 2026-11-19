//! Clone an existing deck.

use uuid::Uuid;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::http::contracts::deck::{HttpCloneDeck, HttpClonedDeck};
use zwipe_core::http::endpoints::deck::CloneDeck;

use crate::{ClientError, ZwipeClient};

impl ZwipeClient {
    /// Clones an existing deck into a new one with a caller-chosen name, under a deck id of its
    /// own: the body's, or a fresh one when it has none, so each call is one copy. When the reply
    /// is lost ([`ClientError::Network`]), the copy is looked up by that id: a clone that landed
    /// answers as cloned, and one that didn't answers the network error.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError`] when the request fails to complete and the copy isn't there,
    /// the server answers with an error status, or the reply doesn't decode.
    pub async fn clone_deck(
        &self,
        source_deck_id: Uuid,
        body: &HttpCloneDeck,
        session: &Session,
    ) -> Result<HttpClonedDeck, ClientError> {
        let id = body.id.unwrap_or_else(Uuid::new_v4);
        let body = HttpCloneDeck {
            id: Some(id),
            ..body.clone()
        };
        match self
            .call(CloneDeck(source_deck_id, body), Some(session))
            .await
        {
            Err(lost @ ClientError::Network(_)) => self
                .fetch_deck_profile(id, session)
                .await
                .map(|copy| HttpClonedDeck { deck_id: copy.id })
                .map_err(|_| lost),
            answered => answered,
        }
    }
}
