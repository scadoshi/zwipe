//! Clear a deck's suppression set (skipped/removed cards).

use uuid::Uuid;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::http::contracts::deck::HttpClearedSuppressions;
use zwipe_core::http::endpoints::deck::ClearDeckSuppressions;

use crate::{ClientError, ZwipeClient};

impl ZwipeClient {
    /// Clears a deck's suppression set.
    pub async fn clear_deck_suppressions(
        &self,
        deck_id: Uuid,
        session: &Session,
    ) -> Result<HttpClearedSuppressions, ClientError> {
        self.call(ClearDeckSuppressions(deck_id), Some(session))
            .await
    }
}
