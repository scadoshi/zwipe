use dioxus::prelude::*;
use crate::{Footer, Nav};

const STRIPE_URL: &str = "https://buy.stripe.com/5kQdRa5tUeNm9pd8BY9Zm00";
const BMC_URL: &str = "https://buymeacoffee.com/scadoshi";
const GITHUB_SPONSORS_URL: &str = "https://github.com/sponsors/scadoshi";

#[component]
pub fn Contribute() -> Element {
    rsx! {
        Nav {}
        div { class: "page",
            div { class: "section",
                h1 { "contribute" }
                p {
                    "zwipe is a solo indie project — designed, built, and shipped by one person.
                    if you're enjoying it, any support goes directly toward server costs and
                    keeping development moving."
                }
            }

            div { class: "section",
                div { class: "card",
                    h3 { "one-time contribution" }
                    p { "pay what you want via stripe. no account required." }
                    a {
                        href: STRIPE_URL,
                        target: "_blank",
                        rel: "noopener noreferrer",
                        class: "appstore-btn",
                        "contribute via stripe ↗"
                    }
                }
                div { class: "card",
                    h3 { "buy me a coffee" }
                    p { "quick one-off support through buy me a coffee." }
                    a {
                        href: BMC_URL,
                        target: "_blank",
                        rel: "noopener noreferrer",
                        class: "appstore-btn",
                        "buymeacoffee.com/scadoshi ↗"
                    }
                }
                div { class: "card",
                    h3 { "github sponsors" }
                    p { "recurring monthly support via github sponsors." }
                    a {
                        href: GITHUB_SPONSORS_URL,
                        target: "_blank",
                        rel: "noopener noreferrer",
                        class: "appstore-btn",
                        "github.com/sponsors/scadoshi ↗"
                    }
                }
            }
        }
        Footer {}
    }
}
