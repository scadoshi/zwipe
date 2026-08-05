//! Fetch the featured flavor card — one shared pick per UTC hour.
//!
//! Unauthed: the home screen shows it pre- and post-login, and the server
//! serves everyone the same card from its in-memory slot (see the featured
//! flavor plan).

use crate::outbound::client::{ClientError, ZwipeClient};
use reqwest::StatusCode;
use std::future::Future;
use tracing::info;
use zwipe::inbound::http::routes::featured_flavor_route;
use zwipe_core::domain::card::Card;

/// Trait for fetching the hour's featured flavor card.
#[allow(missing_docs)]
pub trait ClientFeaturedFlavor {
    fn featured_flavor(&self) -> impl Future<Output = Result<Card, ClientError>> + Send;
}

impl ClientFeaturedFlavor for ZwipeClient {
    async fn featured_flavor(&self) -> Result<Card, ClientError> {
        let mut url = self.app_config.backend_url.clone();
        url.set_path(&featured_flavor_route());
        info!("GET {}", url);

        let response = self.client.get(url).send().await?;

        match response.status() {
            StatusCode::OK => {
                let card: Card = response.json().await?;
                Ok(card)
            }
            status => {
                let message = response.text().await?;
                Err((status, message).into())
            }
        }
    }
}
