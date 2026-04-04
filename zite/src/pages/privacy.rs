use dioxus::prelude::*;
use crate::{Footer, Nav};

#[component]
pub fn Privacy() -> Element {
    rsx! {
        Nav {}
        div { class: "page content-enter",
            div { class: "section",
                h1 { "privacy policy" }
                p { "last updated: march 2026" }
            }

            div { class: "privacy-content section",
                h2 { "overview" }
                p {
                    "zwipe is a magic: the gathering deck builder for mobile. this policy
                    describes what data we collect, how we use it, and your rights."
                }

                h2 { "data we collect" }
                ul {
                    li { strong { "account data" } " — email address, username, and a hashed password (never stored in plaintext)." }
                    li { strong { "deck data" } " — the decks and card selections you create within the app." }
                    li { strong { "session data" } " — authentication tokens stored securely on your device." }
                }
                p {
                    "we do not collect location data, device identifiers, analytics, or any data
                    beyond what is required to operate the app."
                }

                h2 { "how we use your data" }
                ul {
                    li { "to authenticate your account and maintain sessions." }
                    li { "to store and sync your decks across devices." }
                    li { "to send transactional emails (email verification, password reset)." }
                }
                p {
                    "we do not sell, share, or use your data for advertising or analytics."
                }

                h2 { "third-party services" }
                ul {
                    li {
                        strong { "scryfall" }
                        " — card data (names, images, oracle text) is sourced from the scryfall api and stored on our servers. your account data is never shared with scryfall."
                    }
                    li {
                        strong { "resend" }
                        " — transactional email delivery (verification and password reset emails). your email address is passed to resend solely to deliver these messages."
                    }
                }

                h2 { "data retention" }
                p {
                    "your data is retained as long as your account exists. you can delete your
                    account at any time from within the app, which permanently removes all
                    associated data."
                }

                h2 { "security" }
                p {
                    "passwords are hashed with argon2. refresh tokens are sha-256 hashed before
                    storage. all traffic is encrypted in transit via https. we do not have access
                    to your plaintext password."
                }

                h2 { "children" }
                p {
                    "zwipe is not directed at children under 13. we do not knowingly collect
                    data from children under 13."
                }

                h2 { "contact" }
                p {
                    "questions or requests: "
                    a { href: "mailto:support@zwipe.net", "support@zwipe.net" }
                }
            }
        }
        Footer {}
    }
}
