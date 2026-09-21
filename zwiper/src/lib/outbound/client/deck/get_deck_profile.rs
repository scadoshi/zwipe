//! Fetch a single deck profile (metadata only).

use crate::outbound::client::{ClientError, ZwipeClient};
use std::future::Future;
use uuid::Uuid;
use zwipe_core::{
    domain::{auth::models::session::Session, deck::deck_profile::DeckProfile},
    http::endpoints::deck::GetDeckProfile,
};

/// Trait for fetching deck metadata without cards.
#[allow(missing_docs)]
pub trait ClientGetDeckProfile {
    fn get_deck_profile(
        &self,
        deck_id: Uuid,
        session: &Session,
    ) -> impl Future<Output = Result<DeckProfile, ClientError>> + Send;
}

impl ClientGetDeckProfile for ZwipeClient {
    async fn get_deck_profile(
        &self,
        deck_id: Uuid,
        session: &Session,
    ) -> Result<DeckProfile, ClientError> {
        self.call(GetDeckProfile(deck_id), Some(session)).await
    }
}
