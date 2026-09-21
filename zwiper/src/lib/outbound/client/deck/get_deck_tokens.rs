//! Fetch tokens produced by a deck's cards.

use crate::outbound::client::{ClientError, ZwipeClient};
use std::future::Future;
use uuid::Uuid;
use zwipe_core::{
    domain::{auth::models::session::Session, card::Card},
    http::endpoints::deck::GetDeckTokens,
};

/// Trait for fetching all token cards produced by a deck.
#[allow(missing_docs)]
pub trait ClientGetDeckTokens {
    fn get_deck_tokens(
        &self,
        deck_id: Uuid,
        session: &Session,
    ) -> impl Future<Output = Result<Vec<Card>, ClientError>> + Send;
}

impl ClientGetDeckTokens for ZwipeClient {
    async fn get_deck_tokens(
        &self,
        deck_id: Uuid,
        session: &Session,
    ) -> Result<Vec<Card>, ClientError> {
        self.call(GetDeckTokens(deck_id), Some(session)).await
    }
}
