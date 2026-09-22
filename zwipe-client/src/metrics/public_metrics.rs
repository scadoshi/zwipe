//! Public install and deck counts for the site's stats strip.

use crate::{ClientError, ZwipeClient};
use zwipe_core::http::{contracts::metrics::HttpPublicMetrics, endpoints::metrics::PublicMetrics};

impl ZwipeClient {
    /// Fetches the public counts shown on the marketing pages.
    ///
    /// Unauthenticated: it renders for visitors with no account.
    pub async fn public_metrics(&self) -> Result<HttpPublicMetrics, ClientError> {
        self.call(PublicMetrics, None).await
    }
}
