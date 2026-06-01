use dioxus::prelude::*;
use crate::components::PageMeta;
use crate::{Footer, Nav};

#[component]
pub fn Privacy() -> Element {
    rsx! {
        PageMeta {
            title: "Privacy Policy",
            description: "Zwipe privacy policy — what data is collected, how it's used, and your rights.",
            path: "/privacy",
        }
        Nav {}
        div { class: "page content-enter",
            div { class: "section",
                h1 { "Privacy Policy" }
                p { "Last updated: April 2026" }
            }

            div { class: "privacy-content section",
                h2 { "Overview" }
                p {
                    "Zwipe is a "
                    a { href: "https://magic.wizards.com/en", target: "_blank", rel: "noopener noreferrer", "Magic: The Gathering" }
                    " deck builder for mobile. This policy describes what data we collect, how we use it, and your rights."
                }

                h2 { "Data We Collect" }
                ul {
                    li { strong { "Account data" } " — email address, username, and a hashed password (never stored in plaintext)." }
                    li { strong { "Deck data" } " — the decks and card selections you create within the app." }
                    li { strong { "Session data" } " — authentication tokens stored securely on your device." }
                }
                p {
                    "We do not collect location data, device identifiers, analytics, or any data
                    beyond what is required to operate the app."
                }

                h2 { "How We Use Your Data" }
                ul {
                    li { "To authenticate your account and maintain sessions." }
                    li { "To store and sync your decks across devices." }
                    li { "To send transactional emails (email verification, password reset)." }
                }
                p {
                    "We do not sell, share, or use your data for advertising or analytics."
                }

                h2 { "Third-Party Services" }
                ul {
                    li {
                        strong {
                            a { href: "https://scryfall.com", target: "_blank", rel: "noopener noreferrer", "Scryfall" }
                        }
                        " — card data (names, images, oracle text) is sourced from the Scryfall API and stored on our servers. Your account data is never shared with Scryfall."
                    }
                    li {
                        strong {
                            a { href: "https://resend.com", target: "_blank", rel: "noopener noreferrer", "Resend" }
                        }
                        " — transactional email delivery (verification and password reset emails). Your email address is passed to Resend solely to deliver these messages."
                    }
                }

                h2 { "Data Retention" }
                p {
                    "Your data is retained as long as your account exists. You can delete your
                    account at any time from within the app, which permanently removes all
                    associated data."
                }

                h2 { "Security" }
                p {
                    "Passwords are hashed with argon2. Refresh tokens are SHA-256 hashed before
                    storage. All traffic is encrypted in transit via HTTPS. We do not have access
                    to your plaintext password."
                }

                h2 { "Children" }
                p {
                    "Zwipe is not directed at children under 13. We do not knowingly collect
                    data from children under 13."
                }

                h2 { "Contact" }
                p {
                    "Questions or requests? Join the "
                    a { href: "https://discord.gg/s2UReqUUeg", target: "_blank", rel: "noopener noreferrer", "Discord" }
                    " for support."
                }
            }
        }
        Footer {}
    }
}
