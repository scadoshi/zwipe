//! The app's `Help & feedback` dialog.

use components::{AlertDialog, DialogAction};
use dioxus::prelude::*;
use zwipe_core::domain::site::DISCORD_URL;

use crate::config::APP_VERSION;
use crate::domain::support;

/// The dialog, drawn by the root's dialog host while `open` is true.
#[component]
pub fn HelpDialog(open: Signal<bool>) -> Element {
    rsx! {
        AlertDialog {
            open,
            title: "Help & feedback",
            actions: vec![
                DialogAction::Dismiss("Close".into()),
                DialogAction::Link {
                    label: "Report a problem \u{2197}".into(),
                    href: support::report_mailto(APP_VERSION),
                },
                DialogAction::Link {
                    label: "Join the Discord \u{2197}".into(),
                    href: DISCORD_URL.into(),
                },
            ],
            "Reach the team or join the community."
        }
    }
}
