//! Endpoints describing the service itself rather than a user's data.

use crate::http::{
    contracts::{changelog::HttpChangelog, client::HttpMinClientVersion},
    endpoint::{Endpoint, Method},
    paths::{CHANGELOG_ROUTE, MIN_CLIENT_VERSION_ROUTE},
};

/// The release history the app and site both render.
pub struct GetChangelog;
impl Endpoint for GetChangelog {
    const METHOD: Method = Method::Get;
    const AUTH: bool = false;
    type Response = HttpChangelog;
    fn path(&self) -> String {
        CHANGELOG_ROUTE.to_string()
    }
}

/// The force-update gate's minimum client version.
pub struct GetMinClientVersion;
impl Endpoint for GetMinClientVersion {
    const METHOD: Method = Method::Get;
    const AUTH: bool = false;
    type Response = HttpMinClientVersion;
    fn path(&self) -> String {
        MIN_CLIENT_VERSION_ROUTE.to_string()
    }
}
