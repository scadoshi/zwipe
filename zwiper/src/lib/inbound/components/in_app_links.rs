//! Routes every plain link through [`open_url::open`].
//!
//! Dioxus hands a click on an `<a href>` that no handler claimed to the
//! `webbrowser` crate, which leaves the app for Safari or the default browser.
//! That covers links inside shared HTML (the privacy policy's), the
//! `DialogAction::Link` buttons and every `target: "_blank"` anchor. This
//! replaces the interpreter's `handleClickNavigate` with one that sends the
//! `href` here instead, so web pages open in the in-app browser and `mailto:`
//! reaches the mail app. Router links prevent the default in Rust first, so
//! they never reach it.

use dioxus::prelude::*;

use crate::outbound::open_url;

/// Replaces the interpreter's link hook with one that sends each `href` back
/// over this eval's channel. It matches the original: the nearest `<a>`, the
/// default prevented, an empty `href` ignored.
const ROUTE_LINKS_JS: &str = r#"
const interpreter = window.interpreter;
if (interpreter) {
    interpreter.handleClickNavigate = (event, target) => {
        const anchor = target.closest("a");
        if (!anchor) return;
        event.preventDefault();
        const href = anchor.getAttribute("href");
        if (href) dioxus.send(href);
    };
}
"#;

/// Installs the link hook for the life of the calling component. Call it once,
/// from the app root.
pub fn use_in_app_links() {
    use_future(|| async {
        let mut eval = document::eval(ROUTE_LINKS_JS);
        while let Ok(href) = eval.recv::<String>().await {
            open_url::open(&href);
        }
    });
}
