use crate::{Footer, Nav, components::PageMeta};
use dioxus::prelude::*;
use zwipe_components::Panel;

const STRIPE_URL: &str = "https://buy.stripe.com/5kQdRa5tUeNm9pd8BY9Zm00";
const BMC_URL: &str = "https://buymeacoffee.com/scadoshi";
const GITHUB_SPONSORS_URL: &str = "https://github.com/sponsors/scadoshi";

/// Mirrors the portfolio site's contribute page (same shared `Panel` cards,
/// same three options and copy) adapted to zite's chrome and a Zwipe-specific
/// intro.
#[component]
pub fn Contribute() -> Element {
    rsx! {
        PageMeta {
            title: "Contribute",
            description: "Support Zwipe development. Donate via Stripe, Buy Me a Coffee, or GitHub Sponsors. Funds keep the app free and the servers running.",
            path: "/contribute",
        }
        Nav {}
        div { class: "page content-enter",
            div { class: "section panel",
                h1 { "Contribute" }
                p {
                    "Zwipe is a solo indie project: designed, built, and shipped by one person.
                    If you're enjoying it, any support goes directly toward server costs and
                    keeping development moving."
                }
            }

            div { class: "section",
                div { class: "card-grid",
                    Panel {
                        eyebrow: "One-Time",
                        title: "Stripe",
                        actions: rsx! {
                            a {
                                class: "panel-action",
                                href: STRIPE_URL,
                                target: "_blank",
                                rel: "noopener noreferrer",
                                "Contribute \u{2197}"
                            }
                        },
                        p { class: "card-summary", "Pay what you want. No account required." }
                    }
                    Panel {
                        eyebrow: "One-Time",
                        title: "Buy Me a Coffee",
                        actions: rsx! {
                            a {
                                class: "panel-action",
                                href: BMC_URL,
                                target: "_blank",
                                rel: "noopener noreferrer",
                                "Contribute \u{2197}"
                            }
                        },
                        p { class: "card-summary", "Quick one-off support through Buy Me a Coffee." }
                    }
                    Panel {
                        eyebrow: "Recurring",
                        title: "GitHub Sponsors",
                        actions: rsx! {
                            a {
                                class: "panel-action",
                                href: GITHUB_SPONSORS_URL,
                                target: "_blank",
                                rel: "noopener noreferrer",
                                "Contribute \u{2197}"
                            }
                        },
                        p { class: "card-summary", "Recurring monthly support via GitHub Sponsors." }
                    }
                }
            }
        }
        Footer {}
    }
}
