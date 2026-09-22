//! Clear a deck's suppression set (skipped/removed cards).

use crate::{ClientError, ZwipeClient};
use uuid::Uuid;
use zwipe_core::{
    domain::auth::models::session::Session,
    http::{contracts::deck::HttpClearedSuppressions, endpoints::deck::ClearDeckSuppressions},
};

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
