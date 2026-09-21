//! Clear a deck's suppression set (skipped/removed cards).

use crate::outbound::client::{ClientError, ZwipeClient};
use std::future::Future;
use uuid::Uuid;
use zwipe_core::{
    domain::auth::models::session::Session,
    http::{contracts::deck::HttpClearedSuppressions, endpoints::deck::ClearDeckSuppressions},
};

/// Trait for clearing a deck's suppression set.
#[allow(missing_docs)]
pub trait ClientClearDeckSuppressions {
    fn clear_deck_suppressions(
        &self,
        deck_id: Uuid,
        session: &Session,
    ) -> impl Future<Output = Result<HttpClearedSuppressions, ClientError>> + Send;
}

impl ClientClearDeckSuppressions for ZwipeClient {
    async fn clear_deck_suppressions(
        &self,
        deck_id: Uuid,
        session: &Session,
    ) -> Result<HttpClearedSuppressions, ClientError> {
        self.call(ClearDeckSuppressions(deck_id), Some(session))
            .await
    }
}
