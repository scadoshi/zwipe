//! The shared API client, pointed at zwipe.net's backend.
//!
//! zite calls the same endpoints zwiper does, described once in
//! `zwipe-core`'s `Endpoint` impls and sent by `zwipe-client`. Nothing here
//! builds a URL or a request by hand.

use std::sync::OnceLock;

use reqwest::Url;
use zwipe_client::ZwipeClient;

use crate::API_BASE;

/// The process-wide client. Cheap to clone; reqwest pools connections.
#[expect(
    clippy::unwrap_used,
    reason = "API_BASE is a compile-time const: a value that fails to parse breaks every \
              build's first page load, a build bug rather than a runtime condition"
)]
pub(crate) fn client() -> ZwipeClient {
    static CLIENT: OnceLock<ZwipeClient> = OnceLock::new();
    CLIENT
        .get_or_init(|| ZwipeClient::new(Url::parse(API_BASE).unwrap()))
        .clone()
}
