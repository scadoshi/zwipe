//! The browser window itself: opening a page in a new tab.

/// Opens `url` in a new tab, with no handle back to this page. A blocked
/// pop-up is only logged.
pub fn open_in_new_tab(url: &str) {
    #[cfg(target_arch = "wasm32")]
    {
        let opened = web_sys::window()
            .map(|window| window.open_with_url_and_target_and_features(url, "_blank", "noopener"));
        if !matches!(opened, Some(Ok(_))) {
            tracing::warn!("could not open {url}");
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    tracing::debug!("no browser to open {url}");
}
