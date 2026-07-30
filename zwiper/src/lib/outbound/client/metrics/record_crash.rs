//! Crash report POST (no auth — the launch after a crash may have no session).

use crate::outbound::client::{ClientError, ZwipeClient};
use reqwest::StatusCode;
use std::future::Future;
use zwipe_core::http::{contracts::metrics::HttpCrashReport, paths::record_crash_route};

/// Trait for posting a crash report from the previous run.
#[allow(missing_docs)]
pub trait ClientRecordCrash {
    fn record_crash(
        &self,
        report: &HttpCrashReport,
    ) -> impl Future<Output = Result<(), ClientError>> + Send;
}

impl ClientRecordCrash for ZwipeClient {
    async fn record_crash(&self, report: &HttpCrashReport) -> Result<(), ClientError> {
        let mut url = self.app_config.backend_url.clone();
        url.set_path(&record_crash_route());

        let response = self.client.post(url).json(report).send().await?;

        match response.status() {
            StatusCode::NO_CONTENT | StatusCode::OK => Ok(()),
            status => {
                let message = response.text().await?;
                Err((status, message).into())
            }
        }
    }
}
