//! Support / help entry point.
//!
//! A small faded "!" button rendered at the left of every screen header (via
//! [`ScreenHeader`](super::screen_header::ScreenHeader)), mirroring the "?"
//! hint on the right. Tapping it opens a bottom sheet with ways to reach the
//! team: email a problem report or join the Discord. The report email is
//! pre-filled with the app version and platform so every report self-documents.

use crate::{inbound::components::bottom_sheet::BottomSheet, outbound::open_url};
use dioxus::prelude::*;
use zwipe_components::Button;
use zwipe_core::domain::site::{DISCORD_URL, SUPPORT_EMAIL};

/// App version, baked at compile time.
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Platform label for the report email.
#[cfg(target_os = "android")]
const PLATFORM: &str = "Android";
#[cfg(not(target_os = "android"))]
const PLATFORM: &str = "iOS";

/// Button glyph — a plain `!` to mirror the faded `?` hint button.
const GLYPH: &str = "!";

/// Percent-encodes a string for use in a `mailto:` query (RFC 3986 unreserved
/// set passes through; everything else — spaces, newlines, `·` — is encoded).
fn urlencode(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for byte in input.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(byte as char);
            }
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}

/// Floating help button + support bottom sheet, rendered globally.
#[component]
pub fn SupportButton() -> Element {
    let mut open = use_signal(|| false);

    let subject = urlencode("Zwipe problem report");
    let body = urlencode(&format!(
        "Describe the problem:\n\n\n---\nZwipe {APP_VERSION} \u{00b7} {PLATFORM}"
    ));
    let mailto = format!("mailto:{SUPPORT_EMAIL}?subject={subject}&body={body}");

    rsx! {
        button {
            class: "util-btn page-header-support",
            "aria-label": "Help and feedback",
            onclick: move |_| open.set(true),
            "{GLYPH}"
        }

        BottomSheet { open, title: "Help & feedback".to_string(),
            // A plain `<a href="mailto:">` does nothing on mobile: the webview's
            // navigation handler hands it to `webbrowser`, which rejects
            // non-http(s) URLs. Open it through the OS ourselves instead.
            Button {
                onclick: move |_| {
                    open_url::open(&mailto);
                    open.set(false);
                },
                "Report a problem \u{2197}"
            }
            a {
                class: "btn",
                href: "{DISCORD_URL}",
                target: "_blank",
                rel: "noopener noreferrer",
                style: "text-decoration: none; text-align: center;",
                onclick: move |_| open.set(false),
                "Join the Discord \u{2197}"
            }
        }
    }
}
