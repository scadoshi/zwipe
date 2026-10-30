//! Clone an existing deck.

use uuid::Uuid;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::http::contracts::deck::{HttpCloneDeck, HttpClonedDeck};
use zwipe_core::http::endpoints::deck::CloneDeck;

use crate::{ClientError, ZwipeClient};

impl ZwipeClient {
    /// Clones an existing deck into a new one with a caller-chosen name.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError`] when the request fails to complete, the server answers with an
    /// error status, or the reply doesn't decode.
    pub async fn clone_deck(
        &self,
        source_deck_id: Uuid,
        body: &HttpCloneDeck,
        session: &Session,
    ) -> Result<HttpClonedDeck, ClientError> {
        self.call(CloneDeck(source_deck_id, body.clone()), Some(session))
            .await
    }
}
