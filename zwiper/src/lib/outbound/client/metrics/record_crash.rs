//! Crash report POST (no auth: the launch after a crash may have no session).

use crate::outbound::client::{ClientError, ZwipeClient};
use std::future::Future;
use zwipe_core::http::{contracts::metrics::HttpCrashReport, endpoints::metrics::RecordCrash};

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
        let body = serde_json::to_value(report)?;
        self.call(RecordCrash(body), None).await
    }
}
