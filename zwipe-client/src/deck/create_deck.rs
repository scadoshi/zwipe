//! Create new deck.

use crate::{ClientError, ZwipeClient};
use zwipe_core::{
    domain::{auth::models::session::Session, deck::deck_profile::DeckProfile},
    http::{contracts::deck::HttpCreateDeckProfile, endpoints::deck::CreateDeck},
};

impl ZwipeClient {
    /// Creates new deck profiles.
    pub async fn create_deck_profile(
        &self,
        request: &HttpCreateDeckProfile,
        session: &Session,
    ) -> Result<DeckProfile, ClientError> {
        self.call(CreateDeck(request.clone()), Some(session)).await
    }
}
