//! Fetch all card types.

use crate::outbound::client::ZwipeClient;
use reqwest::StatusCode;
use std::future::Future;
use tracing::info;
use zwipe::inbound::http::{ApiError, routes::get_card_types_route};

/// Trait for fetching the list of all card types (creature, instant, etc.).
#[allow(missing_docs)]
pub trait ClientGetCardTypes {
    fn get_card_types(&self) -> impl Future<Output = Result<Vec<String>, ApiError>> + Send;
}

impl ClientGetCardTypes for ZwipeClient {
    async fn get_card_types(&self) -> Result<Vec<String>, ApiError> {
        let mut url = self.app_config.backend_url.clone();
        url.set_path(&get_card_types_route());
        info!("GET {}", url);

        let response = self.client.get(url).send().await?;

        match response.status() {
            StatusCode::OK => {
                let card_types: Vec<String> = response.json().await?;
                Ok(card_types)
            }
            status => {
                let message = response.text().await?;
                Err((status, message).into())
            }
        }
    }
}
