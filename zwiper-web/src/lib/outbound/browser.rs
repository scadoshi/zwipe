//! The browser window itself: its size, leaving for another page, the
//! clipboard, and saving a file.

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

/// Saves `text` as a file named `file_name`, as the browser saves any download: a CSV blob and a
/// click on a link to it, which is then let go.
pub fn download(file_name: &str, text: &str) {
    dioxus::document::eval(&download_js(file_name, text));
}

/// The script [`download`] runs. The name and the text go in as JSON strings, so nothing in them
/// reads as script.
fn download_js(file_name: &str, text: &str) -> String {
    let name = serde_json::to_string(file_name).unwrap_or_default();
    let body = serde_json::to_string(text).unwrap_or_default();
    format!(
        "(() => {{ const url = URL.createObjectURL(new Blob([{body}], {{ type: 'text/csv;charset=utf-8' }})); \
         const link = document.createElement('a'); link.href = url; link.download = {name}; \
         document.body.appendChild(link); link.click(); link.remove(); \
         setTimeout(() => URL.revokeObjectURL(url), 0); }})()"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A name or text holding quotes, a backslash or a closing tag stays a string in the script.
    #[test]
    fn the_download_script_carries_the_file_as_strings() {
        let js = download_js(
            "zwipe \"collection\".csv",
            "Name\n\"Sol Ring\",</script>\\\n",
        );
        assert!(
            js.contains(r#"link.download = "zwipe \"collection\".csv";"#),
            "{js}"
        );
        assert!(
            js.contains(r#"new Blob(["Name\n\"Sol Ring\",</script>\\\n"]"#),
            "{js}"
        );
        assert!(js.contains("URL.revokeObjectURL(url)"), "{js}");
    }
}
