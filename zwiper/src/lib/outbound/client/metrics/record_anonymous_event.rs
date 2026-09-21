//! Pre-auth funnel event POST (no auth: there is no user yet).

use crate::outbound::client::{ClientError, ZwipeClient};
use std::future::Future;
use zwipe_core::http::{
    contracts::metrics::HttpAnonymousEvent, endpoints::metrics::RecordAnonymousEvent,
};

/// Trait for posting a pre-auth funnel event.
#[allow(missing_docs)]
pub trait ClientRecordAnonymousEvent {
    fn record_anonymous_event(
        &self,
        event: &HttpAnonymousEvent,
    ) -> impl Future<Output = Result<(), ClientError>> + Send;
}

impl ClientRecordAnonymousEvent for ZwipeClient {
    async fn record_anonymous_event(&self, event: &HttpAnonymousEvent) -> Result<(), ClientError> {
        let body = serde_json::to_value(event)?;
        self.call(RecordAnonymousEvent(body), None).await
    }
}
