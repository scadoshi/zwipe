//! Batched usage POST.

use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::http::contracts::metrics::HttpUsageBatch;
use zwipe_core::http::endpoints::metrics::RecordUsage;

use crate::{ClientError, ZwipeClient};

impl ZwipeClient {
    /// Posts a batched usage update.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError`] when the request fails to complete, the server answers with an
    /// error status, or the reply doesn't decode.
    pub async fn record_usage(
        &self,
        batch: &HttpUsageBatch,
        session: &Session,
    ) -> Result<(), ClientError> {
        self.call(RecordUsage(batch.clone()), Some(session)).await
    }
}
