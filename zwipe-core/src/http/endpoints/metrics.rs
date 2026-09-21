//! Usage counters, anonymous funnel events and crash reports.

use crate::http::{
    endpoint::{Endpoint, Method},
    paths::{RECORD_ANONYMOUS_EVENT_ROUTE, RECORD_CRASH_ROUTE, RECORD_USAGE_ROUTE},
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
