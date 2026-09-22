//! Fetch the changelog (release history) from the server.

use crate::outbound::client::{ClientError, ZwipeClient};
use zwipe_core::http::{contracts::changelog::HttpChangelog, endpoints::meta::GetChangelog};

impl ZwipeClient {
    /// Fetches the changelog.
    ///
    /// Public and unauthenticated: the changelog is identical for every user and
    /// wanted pre-login. Fetched once when the changelog screen opens; callers fall
    /// back to the copy compiled into the binary if this fails.
    pub async fn get_changelog(&self) -> Result<HttpChangelog, ClientError> {
        self.call(GetChangelog, None).await
    }
}
