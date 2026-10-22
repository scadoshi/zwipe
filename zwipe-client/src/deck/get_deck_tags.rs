//! Fetch the deck-tag catalog.

use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::deck::DeckTagView;
use zwipe_core::http::endpoints::deck::GetDeckTags;

use crate::{ClientError, ZwipeClient};

impl ZwipeClient {
    /// Fetches the full deck-tag catalog (slug, label, description, seed
    /// otags). Authenticated; it lives under the deck routes; the deck-tag picker
    /// (an authed flow) is its consumer.
    pub async fn get_deck_tags(&self, session: &Session) -> Result<Vec<DeckTagView>, ClientError> {
        self.call(GetDeckTags, Some(session)).await
    }
}
