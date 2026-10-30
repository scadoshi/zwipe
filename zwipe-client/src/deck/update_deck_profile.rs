//! Update deck profile metadata.

use uuid::Uuid;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::deck::deck_profile::DeckProfile;
use zwipe_core::http::contracts::deck::HttpUpdateDeckProfile;
use zwipe_core::http::endpoints::deck::UpdateDeckProfile;

use crate::{ClientError, ZwipeClient};

impl ZwipeClient {
    /// Updates deck profile metadata.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError`] when the request fails to complete, the server answers with an
    /// error status, or the reply doesn't decode.
    pub async fn update_deck_profile(
        &self,
        deck_id: Uuid,
        body: &HttpUpdateDeckProfile,
        session: &Session,
    ) -> Result<DeckProfile, ClientError> {
        self.call(UpdateDeckProfile(deck_id, body.clone()), Some(session))
            .await
    }
}
