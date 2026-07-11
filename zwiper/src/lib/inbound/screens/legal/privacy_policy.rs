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
use zwipe_core::{
    domain::site::{DISCORD_URL, SUPPORT_EMAIL},
    legal::{PRIVACY_LAST_UPDATED, PRIVACY_POLICY_HTML},
};

/// Splits the shared policy HTML into one fragment per `<h2>` heading so each
/// section can render inside its own card, matching the panelled look of the
/// other screens. The leading `<h2>` is re-attached to every chunk and an `<hr>`
/// is inserted under the heading to divide it from the body copy.
fn privacy_sections() -> Vec<String> {
    PRIVACY_POLICY_HTML
        .split("<h2>")
        .map(str::trim)
        .filter(|chunk| !chunk.is_empty())
        .map(|chunk| format!("<h2>{chunk}").replacen("</h2>", "</h2>\n<hr>", 1))
        .collect()
}

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

                    for section in privacy_sections() {
                        div { class: "privacy-section", dangerous_inner_html: section }
                    }

                    div { class: "privacy-section",
                        h2 { "Contact" }
                        hr {}
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
