//! Create new deck.

use crate::outbound::client::{ClientError, ZwipeClient};
use std::future::Future;
use zwipe_core::{
    domain::{auth::models::session::Session, deck::deck_profile::DeckProfile},
    http::{contracts::deck::HttpCreateDeckProfile, endpoints::deck::CreateDeck},
};

/// Trait for creating new deck profiles.
#[allow(missing_docs)]
pub trait ClientCreateDeck {
    fn create_deck_profile(
        &self,
        request: &HttpCreateDeckProfile,
        session: &Session,
    ) -> impl Future<Output = Result<DeckProfile, ClientError>> + Send;
}

impl ClientCreateDeck for ZwipeClient {
    async fn create_deck_profile(
        &self,
        request: &HttpCreateDeckProfile,
        session: &Session,
    ) -> Result<DeckProfile, ClientError> {
        let body = serde_json::to_value(request)?;
        self.call(CreateDeck(body), Some(session)).await
    }
}
