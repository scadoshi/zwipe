//! Fetch all card sets.

use crate::outbound::client::{ClientError, ZwipeClient};
use reqwest::StatusCode;
use std::future::Future;
use tracing::info;
use zwipe_core::http::paths::GET_SETS_ROUTE;

/// Trait for fetching the list of all card sets.
#[allow(missing_docs)]
pub trait ClientGetSets {
    fn get_sets(&self) -> impl Future<Output = Result<Vec<String>, ClientError>> + Send;
}

impl ClientGetSets for ZwipeClient {
    async fn get_sets(&self) -> Result<Vec<String>, ClientError> {
        let mut url = self.app_config.backend_url.clone();
        url.set_path(GET_SETS_ROUTE);
        info!("GET {}", url);

        let response = self.client.get(url).send().await?;

        match response.status() {
            StatusCode::OK => {
                let sets: Vec<String> = response.json().await?;
                Ok(sets)
            }
            status => {
                let message = response.text().await?;
                Err((status, message).into())
            }
        }
    }
}
