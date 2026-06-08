//! Batched usage POST.

use crate::outbound::client::ZwipeClient;
use reqwest::StatusCode;
use std::future::Future;
use zwipe::inbound::http::ApiError;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::http::contracts::metrics::HttpUsageBatch;
use zwipe_core::http::paths::record_usage_route;

/// Trait for posting a batched usage update.
#[allow(missing_docs)]
pub trait ClientRecordUsage {
    fn record_usage(
        &self,
        batch: &HttpUsageBatch,
        session: &Session,
    ) -> impl Future<Output = Result<(), ApiError>> + Send;
}

impl ClientRecordUsage for ZwipeClient {
    async fn record_usage(
        &self,
        batch: &HttpUsageBatch,
        session: &Session,
    ) -> Result<(), ApiError> {
        let mut url = self.app_config.backend_url.clone();
        url.set_path(&record_usage_route());

        let response = self
            .client
            .post(url)
            .json(batch)
            .bearer_auth(&*session.access_token.value)
            .send()
            .await?;

        match response.status() {
            StatusCode::NO_CONTENT | StatusCode::OK => Ok(()),
            status => {
                let message = response.text().await?;
                Err((status, message).into())
            }
        }
    }
}
