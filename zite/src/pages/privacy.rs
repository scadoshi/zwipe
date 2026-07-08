use crate::{DISCORD_URL, Footer, Nav, SUPPORT_EMAIL, components::PageMeta};
use dioxus::prelude::*;
use zwipe_core::legal::{PRIVACY_LAST_UPDATED, PRIVACY_POLICY_HTML};

#[component]
pub fn Privacy() -> Element {
    rsx! {
        PageMeta {
            title: "Privacy Policy",
            description: "Zwipe privacy policy: what data is collected, how it's used, and your rights.",
            path: "/privacy",
        }
        Nav {}
        div { class: "page content-enter",
            div { class: "section panel",
                h1 { "Privacy Policy" }
                p { "Last updated: {PRIVACY_LAST_UPDATED}" }
            }

            div { class: "privacy-content section panel",
                div { dangerous_inner_html: PRIVACY_POLICY_HTML }

                h2 { "Contact" }
                p {
                    "Questions or requests? Email "
                    a { href: "mailto:{SUPPORT_EMAIL}", "{SUPPORT_EMAIL}" }
                    " or join the "
                    a { href: "{DISCORD_URL}", target: "_blank", rel: "noopener noreferrer", "Discord" }
                    " for support."
                }
            }
        }
        Footer {}
    }
}
