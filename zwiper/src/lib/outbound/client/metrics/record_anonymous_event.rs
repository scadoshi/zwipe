//! Pre-auth funnel event POST (no auth — there is no user yet).

use crate::outbound::client::ZwipeClient;
use reqwest::StatusCode;
use std::future::Future;
use zwipe::inbound::http::ApiError;
use zwipe_core::http::{
    contracts::metrics::HttpAnonymousEvent, paths::record_anonymous_event_route,
};

/// Trait for posting a pre-auth funnel event.
#[allow(missing_docs)]
pub trait ClientRecordAnonymousEvent {
    fn record_anonymous_event(
        &self,
        event: &HttpAnonymousEvent,
    ) -> impl Future<Output = Result<(), ApiError>> + Send;
}

impl ClientRecordAnonymousEvent for ZwipeClient {
    async fn record_anonymous_event(&self, event: &HttpAnonymousEvent) -> Result<(), ApiError> {
        let mut url = self.app_config.backend_url.clone();
        url.set_path(&record_anonymous_event_route());

        let response = self.client.post(url).json(event).send().await?;

        match response.status() {
            StatusCode::NO_CONTENT | StatusCode::OK => Ok(()),
            status => {
                let message = response.text().await?;
                Err((status, message).into())
            }
        }
    }
}
