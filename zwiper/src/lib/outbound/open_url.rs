//! Opens a URL in the in-app browser or with the OS, logging on failure.
//!
//! The platform work is orangui's [`components::open_url`]: on mobile, `http`
//! and `https` open in `SFSafariViewController` or a Custom Tab, and `mailto:`
//! and the rest go to the OS. Unclaimed link clicks reach it through
//! [`components::use_in_app_links`], installed in `App`.

/// Opens `url` in the in-app browser or with the OS default handler, logging on
/// failure.
pub fn open(url: &str) {
    if let Err(e) = components::open_url(url) {
        tracing::error!("failed to open url {url:?}: {e}");
    }
}
