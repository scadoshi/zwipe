//! Crash report POST (no auth: the launch after a crash may have no session).

use zwipe_core::http::contracts::metrics::HttpCrashReport;
use zwipe_core::http::endpoints::metrics::RecordCrash;

use crate::{ClientError, ZwipeClient};

impl ZwipeClient {
    /// Posts a crash report from the previous run.
    pub async fn record_crash(&self, report: &HttpCrashReport) -> Result<(), ClientError> {
        self.call(RecordCrash(report.clone()), None).await
    }
}
