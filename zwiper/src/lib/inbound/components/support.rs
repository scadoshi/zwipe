//! Support / help entry point.
//!
//! A small "!" button in the main menu's bottom row (see
//! [`menu`](crate::inbound::components::navigation::menu)). Tapping it opens a dialog with ways to reach the
//! team: email a problem report or join the Discord. The report email is
//! pre-filled with the app version and platform so every report self-documents.

use components::{AlertDialog, DialogAction};
use dioxus::prelude::*;
use zwipe_core::domain::site::DISCORD_URL;
use zwipe_ui::support;

use crate::outbound::open_url;

/// App version, baked at compile time.
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Platform label for the report email.
#[cfg(target_os = "android")]
const PLATFORM: &str = "Android";
#[cfg(not(target_os = "android"))]
const PLATFORM: &str = "iOS";

/// The menu's help button and its support dialog.
#[component]
pub fn SupportButton() -> Element {
    let mut open = use_signal(|| false);

    let mailto = support::report_mailto(APP_VERSION, PLATFORM);
    // A plain `<a href="mailto:">` does nothing on mobile: the webview's
    // navigation handler hands it to `webbrowser`, which rejects non-http(s)
    // URLs. Open it through the OS.
    let report = use_callback(move |()| open_url::open(&mailto));

    rsx! {
        button {
            class: "btn-action app-nav-support",
            "aria-label": support::BUTTON_LABEL,
            onclick: move |_| open.set(true),
            {support::GLYPH}
        }

        AlertDialog {
            open,
            title: support::TITLE,
            actions: vec![
                DialogAction::Dismiss(support::CLOSE.into()),
                DialogAction::Confirm {
                    label: support::REPORT.into(),
                    danger: false,
                    on_confirm: report,
                },
                DialogAction::Link {
                    label: support::DISCORD.into(),
                    href: DISCORD_URL.into(),
                },
            ],
            {support::BODY}
        }
    }
}
