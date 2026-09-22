//! The shared API client, pointed at zwipe.net's backend.
//!
//! zite calls the same endpoints zwiper does, described once in
//! `zwipe-core`'s `Endpoint` impls and sent by `zwipe-client`. Nothing here
//! builds a URL or a request by hand.

// The base URL is a compile-time const from zwipe-core. A value that fails to
// parse breaks the first page load of every build rather than for some
// visitors and not others, so it is a build bug, not a runtime condition.
#![allow(clippy::unwrap_used)]

use crate::API_BASE;
use reqwest::Url;
use std::sync::OnceLock;
use zwipe_client::ZwipeClient;

/// The process-wide client. Cheap to clone; reqwest pools connections.
pub fn client() -> ZwipeClient {
    static CLIENT: OnceLock<ZwipeClient> = OnceLock::new();
    CLIENT
        .get_or_init(|| ZwipeClient::new(Url::parse(API_BASE).unwrap()))
        .clone()
}
