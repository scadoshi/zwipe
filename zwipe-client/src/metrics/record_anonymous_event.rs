//! Pre-auth funnel event POST (no auth: there is no user yet).

use crate::{ClientError, ZwipeClient};
use zwipe_core::http::{
    contracts::metrics::HttpAnonymousEvent, endpoints::metrics::RecordAnonymousEvent,
};

impl ZwipeClient {
    /// Posts a pre-auth funnel event.
    pub async fn record_anonymous_event(
        &self,
        event: &HttpAnonymousEvent,
    ) -> Result<(), ClientError> {
        self.call(RecordAnonymousEvent(event.clone()), None).await
    }
}
