//! Privacy policy: the section cards, centered at every width.
//!
//! The copy is zwipe-core's, compiled in, so the app, the website and this
//! page say the same; there is nothing to load.

use dioxus::prelude::*;
use zwipe_core::domain::site::{DISCORD_URL, SUPPORT_EMAIL};
use zwipe_core::legal::{PRIVACY_INTRO_HTML, PRIVACY_LAST_UPDATED};

use crate::domain::privacy::{self, CONTACT};
use crate::inbound::shell::ScreenFrame;

/// The policy, from About. Back is the header's arrow.
#[component]
pub fn PrivacyPolicy() -> Element {
    let sections = privacy::sections();

    rsx! {
        ScreenFrame { title: "Privacy policy",
            description: Some("What data Zwipe collects, how it's used, and your rights".to_string()),
            div { class: "screen-content content-enter",
                div { class: "web-privacy",
                    div { class: "privacy-content",
                        div { class: "privacy-section",
                            div { class: "privacy-head",
                                h2 { "Privacy policy" }
                                span { class: "stat-chip", "Last updated {PRIVACY_LAST_UPDATED}" }
                            }
                            hr {}
                            div { dangerous_inner_html: PRIVACY_INTRO_HTML }
                        }
                        for section in sections {
                            div {
                                key: "{section.heading}",
                                id: privacy::anchor(&section.heading),
                                class: "privacy-section",
                                dangerous_inner_html: section.html,
                            }
                        }
                        div { id: privacy::anchor(CONTACT), class: "privacy-section",
                            h2 { "{CONTACT}" }
                            hr {}
                            p {
                                "Questions or requests? Email "
                                a { class: "privacy-link", href: "mailto:{SUPPORT_EMAIL}", "{SUPPORT_EMAIL}" }
                                " or join the "
                                a {
                                    class: "privacy-link",
                                    href: DISCORD_URL,
                                    "Discord"
                                }
                                " for support."
                            }
                        }
                    }
                }
            }
        }
    }
}
