//! Crash report POST (no auth: the launch after a crash may have no session).

use crate::{ClientError, ZwipeClient};
use zwipe_core::http::{contracts::metrics::HttpCrashReport, endpoints::metrics::RecordCrash};

impl ZwipeClient {
    /// Posts a crash report from the previous run.
    pub async fn record_crash(&self, report: &HttpCrashReport) -> Result<(), ClientError> {
        self.call(RecordCrash(report.clone()), None).await
    }
}
