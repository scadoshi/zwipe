//! Fetch all deck profiles for the current user.

use crate::{ClientError, ZwipeClient};
use zwipe_core::{
    domain::{auth::models::session::Session, deck::deck_profile::DeckProfile},
    http::endpoints::deck::GetDeckProfiles,
};

impl ZwipeClient {
    /// Fetches all deck profiles for the authenticated user.
    pub async fn get_deck_profiles(
        &self,
        session: &Session,
    ) -> Result<Vec<DeckProfile>, ClientError> {
        self.call(GetDeckProfiles, Some(session)).await
    }
}
