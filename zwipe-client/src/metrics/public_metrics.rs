//! Public install and deck counts for the site's stats strip.

use zwipe_core::http::contracts::metrics::HttpPublicMetrics;
use zwipe_core::http::endpoints::metrics::PublicMetrics;

use crate::call::Dated;
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
    pub async fn fetch_public_metrics(&self) -> Result<HttpPublicMetrics, ClientError> {
        self.call(PublicMetrics, None).await
    }

    /// Fetches the public counts with when the copy was made, since an edge
    /// cache holds them for a while: see [`ZwipeClient::call_dated`].
    ///
    /// # Errors
    ///
    /// As [`Self::fetch_public_metrics`].
    pub async fn fetch_public_metrics_dated(
        &self,
    ) -> Result<Dated<HttpPublicMetrics>, ClientError> {
        self.call_dated(PublicMetrics, None).await
    }
}
