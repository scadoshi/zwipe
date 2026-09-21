//! Fetch all deck profiles for the current user.

use crate::outbound::client::{ClientError, ZwipeClient};
use std::future::Future;
use zwipe_core::{
    domain::{auth::models::session::Session, deck::deck_profile::DeckProfile},
    http::endpoints::deck::GetDeckProfiles,
};

/// Trait for fetching all deck profiles for the authenticated user.
#[allow(missing_docs)]
pub trait ClientGetDeckList {
    fn get_deck_profiles(
        &self,
        session: &Session,
    ) -> impl Future<Output = Result<Vec<DeckProfile>, ClientError>> + Send;
}

impl ClientGetDeckList for ZwipeClient {
    async fn get_deck_profiles(&self, session: &Session) -> Result<Vec<DeckProfile>, ClientError> {
        self.call(GetDeckProfiles, Some(session)).await
    }
}
