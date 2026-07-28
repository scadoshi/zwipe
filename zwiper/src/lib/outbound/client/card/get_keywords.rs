//! Fetch all keyword abilities.

use crate::outbound::client::{ClientError, ZwipeClient};
use reqwest::StatusCode;
use std::future::Future;
use tracing::info;
use zwipe::inbound::http::routes::get_keywords_route;

/// Trait for fetching the list of all keyword abilities (flying, trample, etc.).
#[allow(missing_docs)]
pub trait ClientGetKeywords {
    fn get_keywords(&self) -> impl Future<Output = Result<Vec<String>, ClientError>> + Send;
}

impl ClientGetKeywords for ZwipeClient {
    async fn get_keywords(&self) -> Result<Vec<String>, ClientError> {
        let mut url = self.app_config.backend_url.clone();
        url.set_path(&get_keywords_route());
        info!("GET {}", url);

        let response = self.client.get(url).send().await?;

        match response.status() {
            StatusCode::OK => {
                let keywords: Vec<String> = response.json().await?;
                Ok(keywords)
            }
            status => {
                let message = response.text().await?;
                Err((status, message).into())
            }
        }
    }
}
