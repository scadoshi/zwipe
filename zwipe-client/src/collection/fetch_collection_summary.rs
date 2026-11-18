//! Fetch what the caller's collection holds.

use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::http::contracts::collection::HttpCollectionSummary;
use zwipe_core::http::endpoints::collection::GetCollectionSummary;

use crate::{ClientError, ZwipeClient};

impl ZwipeClient {
    /// Fetches how many cards and printings the signed-in user's collection holds.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError`] when the request fails to complete, the server answers with an
    /// error status (an older server has no such route and answers 404), or the reply doesn't
    /// decode.
    pub async fn fetch_collection_summary(
        &self,
        session: &Session,
    ) -> Result<HttpCollectionSummary, ClientError> {
        self.call(GetCollectionSummary, Some(session)).await
    }
}
