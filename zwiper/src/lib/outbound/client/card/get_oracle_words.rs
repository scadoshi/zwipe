//! Fetch all oracle text words.

use crate::outbound::client::ZwipeClient;
use reqwest::StatusCode;
use std::future::Future;
use tracing::info;
use zwipe::inbound::http::{ApiError, routes::get_oracle_words_route};

/// Trait for fetching the list of all normalized oracle text words.
#[allow(missing_docs)]
pub trait ClientGetOracleWords {
    fn get_oracle_words(&self) -> impl Future<Output = Result<Vec<String>, ApiError>> + Send;
}

impl ClientGetOracleWords for ZwipeClient {
    async fn get_oracle_words(&self) -> Result<Vec<String>, ApiError> {
        let mut url = self.app_config.backend_url.clone();
        url.set_path(&get_oracle_words_route());
        info!("GET {}", url);

        let response = self.client.get(url).send().await?;

        match response.status() {
            StatusCode::OK => {
                let words: Vec<String> = response.json().await?;
                Ok(words)
            }
            status => {
                let message = response.text().await?;
                Err((status, message).into())
            }
        }
    }
}
