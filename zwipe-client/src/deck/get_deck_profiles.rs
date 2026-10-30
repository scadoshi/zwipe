//! Fetch all deck profiles for the current user.

use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::deck::deck_profile::DeckProfile;
use zwipe_core::http::endpoints::deck::GetDeckProfiles;

use crate::{ClientError, ZwipeClient};

impl ZwipeClient {
    /// Fetches all deck profiles for the authenticated user.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError`] when the request fails to complete, the server answers with an
    /// error status, or the reply doesn't decode.
    pub async fn get_deck_profiles(
        &self,
        session: &Session,
    ) -> Result<Vec<DeckProfile>, ClientError> {
        self.call(GetDeckProfiles, Some(session)).await
    }
}
