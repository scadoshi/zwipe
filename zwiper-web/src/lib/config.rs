//! Build-time configuration.

use zwipe_core::domain::site::API_BASE;

/// The API the build talks to: `BACKEND_URL` when the build set it, else the
/// site default (the local zerver in a debug build, `api.zwipe.net` in a
/// release one). Cargo rebuilds when the variable changes.
pub const BACKEND_URL: &str = match option_env!("BACKEND_URL") {
    Some(url) => url,
    None => API_BASE,
};

/// The app version, sent with login and checked against the server minimum.
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
