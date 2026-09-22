//! Usage counters, anonymous funnel events and crash reports.

use crate::http::{
    contracts::metrics::HttpPublicMetrics,
    endpoint::{Endpoint, Method},
    paths::{
        PUBLIC_METRICS_ROUTE, RECORD_ANONYMOUS_EVENT_ROUTE, RECORD_CRASH_ROUTE, RECORD_USAGE_ROUTE,
    },
};
use serde_json::Value;

/// Flush a batch of buffered usage counters.
pub struct RecordUsage(pub Value);
impl Endpoint for RecordUsage {
    const METHOD: Method = Method::Post;
    type Response = ();
    fn path(&self) -> String {
        RECORD_USAGE_ROUTE.to_string()
    }
    fn body(&self) -> Option<Value> {
        Some(self.0.clone())
    }
}

/// Record one pre-account funnel event.
pub struct RecordAnonymousEvent(pub Value);
impl Endpoint for RecordAnonymousEvent {
    const METHOD: Method = Method::Post;
    const AUTH: bool = false;
    type Response = ();
    fn path(&self) -> String {
        RECORD_ANONYMOUS_EVENT_ROUTE.to_string()
    }
    fn body(&self) -> Option<Value> {
        Some(self.0.clone())
    }
}

/// Report the crash left on disk by the previous launch.
pub struct RecordCrash(pub Value);
impl Endpoint for RecordCrash {
    const METHOD: Method = Method::Post;
    const AUTH: bool = false;
    type Response = ();
    fn path(&self) -> String {
        RECORD_CRASH_ROUTE.to_string()
    }
    fn body(&self) -> Option<Value> {
        Some(self.0.clone())
    }
}

/// Install and deck counts for the site's stats strip.
///
/// Unauthenticated: it renders on a public marketing page.
pub struct PublicMetrics;
impl Endpoint for PublicMetrics {
    const METHOD: Method = Method::Get;
    const AUTH: bool = false;
    type Response = HttpPublicMetrics;
    fn path(&self) -> String {
        PUBLIC_METRICS_ROUTE.to_string()
    }
}
