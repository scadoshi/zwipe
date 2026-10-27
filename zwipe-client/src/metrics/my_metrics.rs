//! The signed-in user's own lifetime counters.

use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::http::contracts::metrics::HttpLifetimeCounters;
use zwipe_core::http::endpoints::metrics::GetMyMetrics;

use crate::{ClientError, ZwipeClient};

impl ZwipeClient {
    /// Reads the user's lifetime swipes, searches and decks created.
    pub async fn my_metrics(&self, session: &Session) -> Result<HttpLifetimeCounters, ClientError> {
        self.call(GetMyMetrics, Some(session)).await
    }
}
