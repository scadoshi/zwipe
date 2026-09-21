//! Fetch all oracle text words.

use crate::outbound::client::{ClientError, ZwipeClient};
use reqwest::StatusCode;
use std::future::Future;
use tracing::info;
use zwipe_core::http::paths::GET_ORACLE_WORDS_ROUTE;

/// Trait for fetching the list of all normalized oracle text words.
#[allow(missing_docs)]
pub trait ClientGetOracleWords {
    fn get_oracle_words(&self) -> impl Future<Output = Result<Vec<String>, ClientError>> + Send;
}

impl ClientGetOracleWords for ZwipeClient {
    async fn get_oracle_words(&self) -> Result<Vec<String>, ClientError> {
        let mut url = self.app_config.backend_url.clone();
        url.set_path(GET_ORACLE_WORDS_ROUTE);
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
