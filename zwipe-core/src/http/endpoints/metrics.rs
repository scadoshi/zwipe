//! Usage counters, anonymous funnel events and crash reports.

use crate::http::{
    contracts::metrics::{HttpAnonymousEvent, HttpCrashReport, HttpPublicMetrics, HttpUsageBatch},
    endpoint::{Endpoint, Method},
    paths::{
        PUBLIC_METRICS_ROUTE, RECORD_ANONYMOUS_EVENT_ROUTE, RECORD_CRASH_ROUTE, RECORD_USAGE_ROUTE,
    },
};
use std::borrow::Cow;

/// Flush a batch of buffered usage counters.
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
