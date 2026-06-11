//! Fetch the server's minimum supported app version.

use crate::outbound::client::ZwipeClient;
use reqwest::StatusCode;
use std::future::Future;
use tracing::debug;
use zwipe::inbound::http::{routes::min_client_version_route, ApiError};
use zwipe_core::http::contracts::client::HttpMinClientVersion;

/// Trait for fetching the server's minimum supported app version.
///
/// Public and unauthenticated — a gated client must be able to learn it's
/// gated without a valid session. Polled by the upkeep loop, so this logs at
/// debug rather than info.
#[allow(missing_docs)]
pub trait ClientGetMinClientVersion {
    fn get_min_client_version(
        &self,
    ) -> impl Future<Output = Result<HttpMinClientVersion, ApiError>> + Send;
}

impl ClientGetMinClientVersion for ZwipeClient {
    async fn get_min_client_version(&self) -> Result<HttpMinClientVersion, ApiError> {
        let mut url = self.app_config.backend_url.clone();
        url.set_path(&min_client_version_route());
        debug!("GET {}", url);

        let response = self.client.get(url).send().await?;

        match response.status() {
            StatusCode::OK => {
                let result: HttpMinClientVersion = response.json().await?;
                Ok(result)
            }
            status => {
                let message = response.text().await?;
                Err((status, message).into())
            }
        }
    }
}
