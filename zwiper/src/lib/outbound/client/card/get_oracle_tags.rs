//! Fetch the oracle tag catalog.

use crate::outbound::client::{ClientError, ZwipeClient};
use reqwest::StatusCode;
use std::future::Future;
use tracing::info;
use zwipe::inbound::http::routes::get_oracle_tags_route;
use zwipe_core::domain::card::oracle_tag::OracleTag;

/// Trait for fetching the full oracle tag catalog (slug, label, description, parents).
#[allow(missing_docs)]
pub trait ClientGetOracleTags {
    fn get_oracle_tags(&self) -> impl Future<Output = Result<Vec<OracleTag>, ClientError>> + Send;
}

impl ClientGetOracleTags for ZwipeClient {
    async fn get_oracle_tags(&self) -> Result<Vec<OracleTag>, ClientError> {
        let mut url = self.app_config.backend_url.clone();
        url.set_path(&get_oracle_tags_route());
        info!("GET {}", url);

        let response = self.client.get(url).send().await?;

        match response.status() {
            StatusCode::OK => {
                let oracle_tags: Vec<OracleTag> = response.json().await?;
                Ok(oracle_tags)
            }
            status => {
                let message = response.text().await?;
                Err((status, message).into())
            }
        }
    }
}
