//! Privacy Policy screen.
//!
//! Renders the shared policy copy from [`zwipe_core::legal`] via
//! `dangerous_inner_html` so the app and the website stay in sync. Inline
//! `https://` links open externally on device through Dioxus's link handling; the
//! support contact line uses [`open_url::open`] because a raw `mailto:` anchor
//! silently fails inside the mobile webview.

use crate::{inbound::components::screen_header::ScreenHeader, outbound::open_url};
use dioxus::prelude::*;
use zwipe_components::{ActionBar, Button, ButtonVariant};
use zwipe_core::legal::{PRIVACY_LAST_UPDATED, PRIVACY_POLICY_HTML};

const SUPPORT_EMAIL: &str = "support@zwipe.net";
const DISCORD_URL: &str = "https://discord.gg/s2UReqUUeg";

/// Full privacy policy, reachable from the Profile screen.
#[component]
pub fn PrivacyPolicy() -> Element {
    let navigator = use_navigator();

    rsx! {
        div { class: "screen",
            ScreenHeader { title: "Privacy Policy" }

            div { class: "screen-content content-enter",
                div { class: "privacy-content",
                    p { class: "privacy-updated", "Last updated: {PRIVACY_LAST_UPDATED}" }
                    div { dangerous_inner_html: PRIVACY_POLICY_HTML }

                    h2 { "Contact" }
                    p {
                        "Questions or requests? Email "
                        span {
                            class: "privacy-link",
                            onclick: move |_| open_url::open(&format!("mailto:{SUPPORT_EMAIL}")),
                            "{SUPPORT_EMAIL}"
                        }
                        " or join the "
                        span {
                            class: "privacy-link",
                            onclick: move |_| open_url::open(DISCORD_URL),
                            "Discord"
                        }
                        " for support."
                    }
                }
            }

            ActionBar {
                Button {
                    variant: ButtonVariant::Util,
                    onclick: move |_| navigator.go_back(),
                    "Back"
                }
            }
        }
    }
}
