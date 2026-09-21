//! Batched usage POST.

use crate::outbound::client::{ClientError, ZwipeClient};
use std::future::Future;
use zwipe_core::{
    domain::auth::models::session::Session,
    http::{contracts::metrics::HttpUsageBatch, endpoints::metrics::RecordUsage},
};

/// Trait for posting a batched usage update.
#[allow(missing_docs)]
pub trait ClientRecordUsage {
    fn record_usage(
        &self,
        batch: &HttpUsageBatch,
        session: &Session,
    ) -> impl Future<Output = Result<(), ClientError>> + Send;
}

impl ClientRecordUsage for ZwipeClient {
    async fn record_usage(
        &self,
        batch: &HttpUsageBatch,
        session: &Session,
    ) -> Result<(), ClientError> {
        let body = serde_json::to_value(batch)?;
        self.call(RecordUsage(body), Some(session)).await
    }
}
