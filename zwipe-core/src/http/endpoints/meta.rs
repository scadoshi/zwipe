//! Endpoints describing the service itself rather than a user's data.

use crate::http::{
    contracts::{changelog::HttpChangelog, client::HttpMinClientVersion},
    endpoint::{Endpoint, Method},
    paths::{CHANGELOG_ROUTE, MIN_CLIENT_VERSION_ROUTE},
};
use std::borrow::Cow;

/// The release history the app and site both render.
pub struct GetChangelog;
impl Endpoint for GetChangelog {
    const METHOD: Method = Method::Get;
    const AUTH: bool = false;
    type Request = ();
    type Response = HttpChangelog;
    fn path(&self) -> Cow<'static, str> {
        Cow::Borrowed(CHANGELOG_ROUTE)
    }
}

/// The force-update gate's minimum client version.
pub struct GetMinClientVersion;
impl Endpoint for GetMinClientVersion {
    const METHOD: Method = Method::Get;
    const AUTH: bool = false;
    type Request = ();
    type Response = HttpMinClientVersion;
    fn path(&self) -> Cow<'static, str> {
        Cow::Borrowed(MIN_CLIENT_VERSION_ROUTE)
    }
}
