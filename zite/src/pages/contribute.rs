use crate::{Footer, Nav, components::PageMeta};
use dioxus::prelude::*;

const STRIPE_URL: &str = "https://buy.stripe.com/5kQdRa5tUeNm9pd8BY9Zm00";
const BMC_URL: &str = "https://buymeacoffee.com/scadoshi";
const GITHUB_SPONSORS_URL: &str = "https://github.com/sponsors/scadoshi";

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
                    div { class: "card",
                        span { class: "card-category", "One-Time" }
                        h3 { class: "card-title", "Stripe" }
                        p { class: "card-summary", "Pay what you want. No account required." }
                        a {
                            href: STRIPE_URL,
                            target: "_blank",
                            rel: "noopener noreferrer",
                            class: "card-action",
                            "Contribute ↗"
                        }
                    }
                    div { class: "card",
                        span { class: "card-category", "One-Time" }
                        h3 { class: "card-title", "Buy Me a Coffee" }
                        p { class: "card-summary", "Quick one-off support." }
                        a {
                            href: BMC_URL,
                            target: "_blank",
                            rel: "noopener noreferrer",
                            class: "card-action",
                            "buymeacoffee.com/scadoshi ↗"
                        }
                    }
                    div { class: "card",
                        span { class: "card-category", "Recurring" }
                        h3 { class: "card-title", "GitHub Sponsors" }
                        p { class: "card-summary", "Monthly support via GitHub." }
                        a {
                            href: GITHUB_SPONSORS_URL,
                            target: "_blank",
                            rel: "noopener noreferrer",
                            class: "card-action",
                            "github.com/sponsors/scadoshi ↗"
                        }
                    }
                }
            }
        }
        Footer {}
    }
}
