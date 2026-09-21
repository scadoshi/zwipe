//! Fetch the server's minimum supported app version.

use crate::outbound::client::{ClientError, ZwipeClient};
use std::future::Future;
use zwipe_core::http::{
    contracts::client::HttpMinClientVersion, endpoints::meta::GetMinClientVersion,
};

/// Trait for fetching the server's minimum supported app version.
///
/// Public and unauthenticated: a gated client must be able to learn it's
/// gated without a valid session. Polled by the upkeep loop, so this logs at
/// debug rather than info.
#[allow(missing_docs)]
pub trait ClientGetMinClientVersion {
    fn get_min_client_version(
        &self,
    ) -> impl Future<Output = Result<HttpMinClientVersion, ClientError>> + Send;
}

impl ClientGetMinClientVersion for ZwipeClient {
    async fn get_min_client_version(&self) -> Result<HttpMinClientVersion, ClientError> {
        self.call(GetMinClientVersion, None).await
    }
}
