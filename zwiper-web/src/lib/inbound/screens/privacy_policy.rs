//! Privacy policy: a sticky contents column beside the section cards on a
//! wide window, the cards alone below the breakpoint.
//!
//! The copy is zwipe-core's, compiled in, so the app, the website and this
//! page say the same; there is nothing to load.

use components::{ActionBar, Button, ButtonVariant};
use dioxus::prelude::*;
use zwipe_core::domain::site::{DISCORD_URL, SUPPORT_EMAIL};
use zwipe_core::legal::{PRIVACY_INTRO_HTML, PRIVACY_LAST_UPDATED};

use crate::domain::privacy::{self, CONTACT};
use crate::inbound::shell::ScreenFrame;

/// The policy, from Account. A contents entry scrolls its card into view.
#[component]
pub fn PrivacyPolicy() -> Element {
    let navigator = use_navigator();
    let sections = privacy::sections();
    let headings = sections
        .iter()
        .map(|s| s.heading.clone())
        .chain([CONTACT.to_string()]);

    rsx! {
        ScreenFrame { title: "Privacy policy",
            div { class: "screen-content content-enter",
                div { class: "web-privacy",
                    nav { class: "web-privacy-contents", "aria-label": "Contents",
                        for heading in headings {
                            button {
                                key: "{heading}",
                                class: "web-privacy-contents-link",
                                onclick: {
                                    let id = privacy::anchor(&heading);
                                    move |_| {
                                        document::eval(&format!(
                                            "document.getElementById('{id}')?.scrollIntoView({{ behavior: 'smooth', block: 'start' }});"
                                        ));
                                    }
                                },
                                "{heading}"
                            }
                        }
                    }
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
                                    target: "_blank",
                                    rel: "noopener noreferrer",
                                    "Discord"
                                }
                                " for support."
                            }
                        }
                    }
                }
            }
            ActionBar {
                Button {
                    variant: ButtonVariant::Action,
                    onclick: move |_| navigator.go_back(),
                    "Back"
                }
            }
        }
    }
}
