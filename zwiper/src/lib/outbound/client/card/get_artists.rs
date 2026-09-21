//! Fetch all unique artist names.

use crate::outbound::client::{ClientError, ZwipeClient};
use reqwest::StatusCode;
use std::future::Future;
use tracing::info;
use zwipe_core::http::paths::GET_ARTISTS_ROUTE;

/// Trait for fetching the list of all unique card artists.
#[allow(missing_docs)]
pub trait ClientGetArtists {
    fn get_artists(&self) -> impl Future<Output = Result<Vec<String>, ClientError>> + Send;
}

impl ClientGetArtists for ZwipeClient {
    async fn get_artists(&self) -> Result<Vec<String>, ClientError> {
        let mut url = self.app_config.backend_url.clone();
        url.set_path(GET_ARTISTS_ROUTE);
        info!("GET {}", url);

        let response = self.client.get(url).send().await?;

        match response.status() {
            StatusCode::OK => {
                let artists: Vec<String> = response.json().await?;
                Ok(artists)
            }
            status => {
                let message = response.text().await?;
                Err((status, message).into())
            }
        }
    }
}
