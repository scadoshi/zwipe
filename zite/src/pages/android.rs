use dioxus::prelude::*;
use crate::components::PageMeta;
use crate::{Footer, Nav};

/// Public tester group — anyone can join, which makes them eligible for the
/// closed test. The bare opt-in/store links below do nothing until you're a
/// member, so this is step 1.
const GROUP_URL: &str = "https://groups.google.com/g/zwipers";
/// Closed-test opt-in page (only works once you've joined the group).
const OPT_IN_URL: &str = "https://play.google.com/apps/testing/com.scadoshi.zwipe";
/// Play Store listing (installs the test build once you're opted in).
const PLAY_URL: &str = "https://play.google.com/store/apps/details?id=com.scadoshi.zwipe";

/// `/download/android` — during the closed beta this page hosts the tester
/// opt-in instructions. Once Zwipe reaches production on Google Play, replace
/// the body with a redirect to `PLAY_URL` (mirror `ios.rs`); every marketing
/// channel points here, so the swap is a one-place change.
#[component]
pub fn Android() -> Element {
    rsx! {
        PageMeta {
            title: "Download for Android",
            description: "Zwipe for Android: swipe-based Magic: The Gathering deck builder, now in open beta on Google Play. Commander support, 110k+ cards, account-synced decks.",
            path: "/download/android",
        }
        Nav {}
        div { class: "page content-enter",
            div { class: "section panel",
                h1 {
                    "Android: "
                    span { class: "accent-tertiary", "Open beta" }
                }
                p {
                    "Zwipe is live on Google Play as a beta while we finish Google's
                    required testing period. Anyone can join — it takes about a minute,
                    in three steps:"
                }
                ol { class: "beta-steps",
                    li {
                        p {
                            strong { "Join the tester group." }
                            " This is what makes you eligible — the links below won't
                            work until you're a member. Use the same Google account
                            that's on your Android phone."
                        }
                        a {
                            href: "{GROUP_URL}",
                            target: "_blank",
                            rel: "noopener noreferrer",
                            class: "store-btn",
                            "Join the group ↗"
                        }
                    }
                    li {
                        p {
                            strong { "Opt in as a tester." }
                            " Open this on the same account and tap “Become a tester.”"
                        }
                        a {
                            href: "{OPT_IN_URL}",
                            target: "_blank",
                            rel: "noopener noreferrer",
                            class: "store-btn",
                            "Opt in ↗"
                        }
                    }
                    li {
                        p {
                            strong { "Install Zwipe." }
                            " Open the Play Store listing and install. (If it says the
                            app isn’t available, give it a few minutes after opting in,
                            then refresh.)"
                        }
                        a {
                            href: "{PLAY_URL}",
                            target: "_blank",
                            rel: "noopener noreferrer",
                            class: "store-btn",
                            "Open on Google Play ↗"
                        }
                    }
                }
                p {
                    "One small favor: please keep Zwipe installed for about two weeks.
                    Google requires testers to stay opted in for 14 days before we can
                    launch publicly, so it genuinely helps us ship."
                }
                p {
                    "On iPhone instead? "
                    a {
                        href: "/download/ios",
                        class: "store-btn",
                        "Download for iOS ↗"
                    }
                }
            }
        }
        Footer {}
    }
}
