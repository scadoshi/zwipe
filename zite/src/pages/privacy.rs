use crate::{DISCORD_URL, Footer, Nav, SUPPORT_EMAIL, components::PageMeta};
use dioxus::prelude::*;
use zwipe_core::legal::{PRIVACY_LAST_UPDATED, PRIVACY_POLICY_HTML};

/// Splits the shared policy HTML into one fragment per `<h2>` heading so each
/// section renders inside its own panel with a divider under the heading,
/// matching the mobile app. The leading `<h2>` is re-attached to every chunk
/// and an `<hr>` inserted under the heading.
fn privacy_sections() -> Vec<String> {
    PRIVACY_POLICY_HTML
        .split("<h2>")
        .map(str::trim)
        .filter(|chunk| !chunk.is_empty())
        .map(|chunk| format!("<h2>{chunk}").replacen("</h2>", "</h2>\n<hr>", 1))
        .collect()
}

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

            div { class: "privacy-content section",
                for section in privacy_sections() {
                    div { class: "privacy-section panel", dangerous_inner_html: section }
                }

                div { class: "privacy-section panel",
                    h2 { "Contact" }
                    hr {}
                    p {
                        "Questions or requests? Email "
                        a { href: "mailto:{SUPPORT_EMAIL}", "{SUPPORT_EMAIL}" }
                        " or join the "
                        a { href: "{DISCORD_URL}", target: "_blank", rel: "noopener noreferrer", "Discord" }
                        " for support."
                    }
                }
            }
        }
        Footer {}
    }
}
