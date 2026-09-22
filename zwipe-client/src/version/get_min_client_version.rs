//! Fetch the server's minimum supported app version.

use crate::{ClientError, ZwipeClient};
use zwipe_core::http::{
    contracts::client::HttpMinClientVersion, endpoints::meta::GetMinClientVersion,
};

impl ZwipeClient {
    /// Fetches the server's minimum supported app version.
    ///
    /// Public and unauthenticated: a gated client must be able to learn it's
    /// gated without a valid session. Polled by the upkeep loop, so this logs at
    /// debug rather than info.
    pub async fn get_min_client_version(&self) -> Result<HttpMinClientVersion, ClientError> {
        self.call(GetMinClientVersion, None).await
    }
}
