//! Fetch the changelog (release history) from the server.

use crate::outbound::client::ZwipeClient;
use reqwest::StatusCode;
use std::future::Future;
use tracing::debug;
use zwipe::inbound::http::{ApiError, routes::changelog_route};
use zwipe_core::http::contracts::changelog::HttpChangelog;

/// Trait for fetching the changelog.
///
/// Public and unauthenticated — the changelog is identical for every user and
/// wanted pre-login. Fetched once when the changelog screen opens; callers fall
/// back to the copy compiled into the binary if this fails.
#[allow(missing_docs)]
pub trait ClientGetChangelog {
    fn get_changelog(&self) -> impl Future<Output = Result<HttpChangelog, ApiError>> + Send;
}

impl ClientGetChangelog for ZwipeClient {
    async fn get_changelog(&self) -> Result<HttpChangelog, ApiError> {
        let mut url = self.app_config.backend_url.clone();
        url.set_path(&changelog_route());
        debug!("GET {}", url);

        let response = self.client.get(url).send().await?;

        match response.status() {
            StatusCode::OK => {
                let result: HttpChangelog = response.json().await?;
                Ok(result)
            }
            status => {
                let message = response.text().await?;
                Err((status, message).into())
            }
        }
    }
}
