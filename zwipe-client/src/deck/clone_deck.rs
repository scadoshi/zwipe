//! Clone an existing deck.

use crate::{ClientError, ZwipeClient};
use uuid::Uuid;
use zwipe_core::{
    domain::auth::models::session::Session,
    http::{
        contracts::deck::{HttpCloneDeck, HttpClonedDeck},
        endpoints::deck::CloneDeck,
    },
};

impl ZwipeClient {
    /// Clones an existing deck into a new one with a caller-chosen name.
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
