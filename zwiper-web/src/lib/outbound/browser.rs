//! The browser window itself: its size, leaving for another page, and the
//! clipboard.

/// The window's inner width and height in CSS pixels, or `None` off the
/// browser.
#[must_use]
pub fn window_size() -> Option<(f64, f64)> {
    #[cfg(target_arch = "wasm32")]
    {
        let window = web_sys::window()?;
        let width = window.inner_width().ok()?.as_f64()?;
        let height = window.inner_height().ok()?.as_f64()?;
        Some((width, height))
    }
    #[cfg(not(target_arch = "wasm32"))]
    None
}

/// Loads `url` in this tab, in place of the app. A refusal is only logged.
pub fn open_here(url: &str) {
    #[cfg(target_arch = "wasm32")]
    {
        let opened = web_sys::window().map(|window| window.location().assign(url));
        if !matches!(opened, Some(Ok(()))) {
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
