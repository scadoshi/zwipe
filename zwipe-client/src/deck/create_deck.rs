//! Create new deck.

use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::deck::deck_profile::DeckProfile;
use zwipe_core::http::contracts::deck::HttpCreateDeckProfile;
use zwipe_core::http::endpoints::deck::CreateDeck;

use crate::{ClientError, ZwipeClient};

impl ZwipeClient {
    /// Creates new deck profiles.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError`] when the request fails to complete, the server answers with an
    /// error status, or the reply doesn't decode.
    pub async fn create_deck_profile(
        &self,
        request: &HttpCreateDeckProfile,
        session: &Session,
    ) -> Result<DeckProfile, ClientError> {
        self.call(CreateDeck(request.clone()), Some(session)).await
    }
}
