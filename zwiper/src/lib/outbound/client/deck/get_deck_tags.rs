//! Fetch the deck-tag catalog.

use crate::outbound::client::{ClientError, ZwipeClient};
use std::future::Future;
use zwipe_core::{
    domain::{auth::models::session::Session, deck::DeckTagView},
    http::endpoints::deck::GetDeckTags,
};

/// Trait for fetching the full deck-tag catalog (slug, label, description, seed
/// otags). Authenticated; it lives under the deck routes; the deck-tag picker
/// (an authed flow) is its consumer.
#[allow(missing_docs)]
pub trait ClientGetDeckTags {
    fn get_deck_tags(
        &self,
        session: &Session,
    ) -> impl Future<Output = Result<Vec<DeckTagView>, ClientError>> + Send;
}

impl ClientGetDeckTags for ZwipeClient {
    async fn get_deck_tags(&self, session: &Session) -> Result<Vec<DeckTagView>, ClientError> {
        self.call(GetDeckTags, Some(session)).await
    }
}
