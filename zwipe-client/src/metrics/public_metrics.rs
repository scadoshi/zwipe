//! Public install and deck counts for the site's stats strip.

use zwipe_core::http::contracts::metrics::HttpPublicMetrics;
use zwipe_core::http::endpoints::metrics::PublicMetrics;

use crate::{ClientError, ZwipeClient};

impl ZwipeClient {
    /// Fetches the public counts shown on the marketing pages.
    ///
    /// Unauthenticated: it renders for visitors with no account.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError`] when the request fails to complete, the server answers with an
    /// error status, or the reply doesn't decode.
    pub async fn public_metrics(&self) -> Result<HttpPublicMetrics, ClientError> {
        self.call(PublicMetrics, None).await
    }
}
