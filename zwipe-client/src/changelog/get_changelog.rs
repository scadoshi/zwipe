//! Fetch the changelog (release history) from the server.

use zwipe_core::http::contracts::changelog::HttpChangelog;
use zwipe_core::http::endpoints::meta::GetChangelog;

use crate::{ClientError, ZwipeClient};

impl ZwipeClient {
    /// Fetches the changelog.
    ///
    /// Public and unauthenticated: the changelog is identical for every user and
    /// wanted pre-login. Fetched once when the changelog screen opens; callers fall
    /// back to the copy compiled into the binary if this fails.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError`] when the request fails to complete, the server answers with an
    /// error status, or the reply doesn't decode.
    pub async fn get_changelog(&self) -> Result<HttpChangelog, ClientError> {
        self.call(GetChangelog, None).await
    }
}
