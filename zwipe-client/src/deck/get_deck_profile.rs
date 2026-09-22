//! Fetch a single deck profile (metadata only).

use crate::{ClientError, ZwipeClient};
use uuid::Uuid;
use zwipe_core::{
    domain::{auth::models::session::Session, deck::deck_profile::DeckProfile},
    http::endpoints::deck::GetDeckProfile,
};

impl ZwipeClient {
    /// Fetches deck metadata without cards.
    pub async fn get_deck_profile(
        &self,
        deck_id: Uuid,
        session: &Session,
    ) -> Result<DeckProfile, ClientError> {
        self.call(GetDeckProfile(deck_id), Some(session)).await
    }
}
