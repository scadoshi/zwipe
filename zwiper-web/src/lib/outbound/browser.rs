//! The browser window itself: opening a page in a new tab, and the
//! clipboard.

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

/// Puts `text` on the clipboard, as the app does, through the page's
/// `navigator.clipboard`. A refusal, such as from a page without focus, is
/// logged to the console.
pub fn copy_to_clipboard(text: &str) {
    let quoted = serde_json::to_string(text).unwrap_or_default();
    dioxus::document::eval(&format!(
        "navigator.clipboard.writeText({quoted}).catch(e => console.warn('clipboard write refused: ' + e))"
    ));
}
