use dioxus::{document::eval, prelude::*};

use crate::{Footer, Nav, components::PageMeta};

/// Live Google Play listing for Zwipe (Android). Locale-adaptive; no `&hl`.
const PLAY_STORE_URL: &str = "https://play.google.com/store/apps/details?id=com.scadoshi.zwipe";

/// `/download/android`: immediately redirects to Google Play. The mobile app and
/// any marketing can point here permanently; the destination is controlled from
/// the site (a push), never from an app update. Mirrors `ios.rs`.
#[component]
pub fn Android() -> Element {
    // Belt-and-suspenders: the meta-refresh below fires from the prerendered
    // HTML before wasm hydration (and for no-JS clients); this eval covers the
    // hydrated client.
    use_effect(|| {
        spawn(async {
            let _ = eval(&format!("window.location.replace('{PLAY_STORE_URL}');")).await;
        });
    });

    rsx! {
        PageMeta {
            title: "Download for Android",
            description: "Zwipe on Google Play: a swipe-based deck builder for trading-card games.",
            path: "/download/android",
        }
        document::Meta { http_equiv: "refresh", content: "0; url={PLAY_STORE_URL}" }
        Nav {}
        div { class: "page content-enter",
            div { class: "section panel",
                h1 { "Opening Google Play…" }
                p {
                    "If you're not redirected automatically, "
                    a {
                        href: "{PLAY_STORE_URL}",
                        target: "_blank",
                        rel: "noopener noreferrer",
                        class: "store-btn",
                        "get Zwipe on Google Play ↗"
                    }
                }
            }
        }
        Footer {}
    }
}
