//! Pre-auth funnel event POST (no auth: there is no user yet).

use zwipe_core::http::contracts::metrics::HttpAnonymousEvent;
use zwipe_core::http::endpoints::metrics::RecordAnonymousEvent;

use crate::{ClientError, ZwipeClient};

impl ZwipeClient {
    /// Posts a pre-auth funnel event.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError`] when the request fails to complete, the server answers with an
    /// error status, or the reply doesn't decode.
    pub async fn record_anonymous_event(
        &self,
        event: &HttpAnonymousEvent,
    ) -> Result<(), ClientError> {
        self.call(RecordAnonymousEvent(event.clone()), None).await
    }
}
