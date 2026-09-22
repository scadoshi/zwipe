//! Batched usage POST.

use crate::{ClientError, ZwipeClient};
use zwipe_core::{
    domain::auth::models::session::Session,
    http::{contracts::metrics::HttpUsageBatch, endpoints::metrics::RecordUsage},
};

impl ZwipeClient {
    /// Posts a batched usage update.
    pub async fn record_usage(
        &self,
        batch: &HttpUsageBatch,
        session: &Session,
    ) -> Result<(), ClientError> {
        self.call(RecordUsage(batch.clone()), Some(session)).await
    }
}
