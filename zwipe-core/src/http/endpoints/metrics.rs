//! Usage counters, anonymous funnel events and crash reports.

use std::borrow::Cow;

use crate::http::contracts::metrics::{
    HttpAnonymousEvent, HttpCrashReport, HttpLifetimeCounters, HttpPublicMetrics, HttpUsageBatch,
};
use crate::http::endpoint::{Endpoint, Method};
use crate::http::paths::{
    GET_MY_METRICS_ROUTE, PUBLIC_METRICS_ROUTE, RECORD_ANONYMOUS_EVENT_ROUTE, RECORD_CRASH_ROUTE,
    RECORD_USAGE_ROUTE,
};

/// Flush a batch of buffered usage counters.
#[derive(Debug)]
pub struct RecordUsage(pub HttpUsageBatch);
impl Endpoint for RecordUsage {
    const METHOD: Method = Method::Post;
    type Request = HttpUsageBatch;
    type Response = ();
    fn path(&self) -> Cow<'static, str> {
        Cow::Borrowed(RECORD_USAGE_ROUTE)
    }
    fn body(&self) -> Option<&Self::Request> {
        Some(&self.0)
    }
}

/// Record one pre-account funnel event.
#[derive(Debug)]
pub struct RecordAnonymousEvent(pub HttpAnonymousEvent);
impl Endpoint for RecordAnonymousEvent {
    const METHOD: Method = Method::Post;
    const AUTH: bool = false;
    type Request = HttpAnonymousEvent;
    type Response = ();
    fn path(&self) -> Cow<'static, str> {
        Cow::Borrowed(RECORD_ANONYMOUS_EVENT_ROUTE)
    }
    fn body(&self) -> Option<&Self::Request> {
        Some(&self.0)
    }
}

/// Report the crash left on disk by the previous launch.
#[derive(Debug)]
pub struct RecordCrash(pub HttpCrashReport);
impl Endpoint for RecordCrash {
    const METHOD: Method = Method::Post;
    const AUTH: bool = false;
    type Request = HttpCrashReport;
    type Response = ();
    fn path(&self) -> Cow<'static, str> {
        Cow::Borrowed(RECORD_CRASH_ROUTE)
    }
    fn body(&self) -> Option<&Self::Request> {
        Some(&self.0)
    }
}

/// Install and deck counts for the site's stats strip.
///
/// Unauthenticated: it renders on a public marketing page.
#[derive(Debug)]
pub struct PublicMetrics;
impl Endpoint for PublicMetrics {
    const METHOD: Method = Method::Get;
    const AUTH: bool = false;
    type Request = ();
    type Response = HttpPublicMetrics;
    fn path(&self) -> Cow<'static, str> {
        Cow::Borrowed(PUBLIC_METRICS_ROUTE)
    }
}

/// The caller's own lifetime counters, the per-user rows the public counts
/// sum.
#[derive(Debug)]
pub struct GetMyMetrics;
impl Endpoint for GetMyMetrics {
    const METHOD: Method = Method::Get;
    type Request = ();
    type Response = HttpLifetimeCounters;
    fn path(&self) -> Cow<'static, str> {
        Cow::Borrowed(GET_MY_METRICS_ROUTE)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn my_metrics_reads_the_user_route() {
        assert_eq!(GetMyMetrics.path(), "/api/user/metrics");
    }
}
