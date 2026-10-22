//! Fetch a single deck profile (metadata only).

use uuid::Uuid;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::deck::deck_profile::DeckProfile;
use zwipe_core::http::endpoints::deck::GetDeckProfile;

use crate::{ClientError, ZwipeClient};

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
