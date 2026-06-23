use dioxus::document::eval;
use dioxus::prelude::*;

use crate::components::PageMeta;
use crate::{Footer, Nav};

/// Live App Store listing for Zwipe (iOS).
const APP_STORE_URL: &str = "https://apps.apple.com/us/app/zwipe-tcg/id6761341603";

/// `/download/ios` — immediately redirects to the App Store. The mobile app and
/// any marketing can point here permanently; the destination is controlled from
/// the site (a push), never from an app update.
#[component]
pub fn Ios() -> Element {
    // Belt-and-suspenders: the meta-refresh below fires from the prerendered
    // HTML before wasm hydration (and for no-JS clients); this eval covers the
    // hydrated client.
    use_effect(|| {
        spawn(async {
            let _ = eval(&format!("window.location.replace('{APP_STORE_URL}');")).await;
        });
    });

    rsx! {
        PageMeta {
            title: "Download for iOS",
            description: "Zwipe on the App Store — a swipe-based deck builder for trading-card games.",
            path: "/download/ios",
        }
        document::Meta { http_equiv: "refresh", content: "0; url={APP_STORE_URL}" }
        Nav {}
        div { class: "page content-enter",
            div { class: "section panel",
                h1 { "Opening the App Store…" }
                p {
                    "If you're not redirected automatically, "
                    a {
                        href: "{APP_STORE_URL}",
                        target: "_blank",
                        rel: "noopener noreferrer",
                        class: "store-btn",
                        "open Zwipe on the App Store ↗"
                    }
                }
            }
        }
        Footer {}
    }
}
