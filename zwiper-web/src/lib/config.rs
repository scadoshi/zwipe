//! Build-time configuration.

use zwipe_core::domain::site::API_BASE;

/// The API the build talks to: `BACKEND_URL` from the process env or this
/// crate's `.env` (`build.rs`), else the site default (the local zerver in a
/// debug build, `api.zwipe.net` in a release one). Cargo rebuilds when either
/// changes.
pub const BACKEND_URL: &str = match option_env!("BACKEND_URL") {
    Some(url) => url,
    None => API_BASE,
};

/// The app version, sent with login and checked against the server minimum.
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
