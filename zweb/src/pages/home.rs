use dioxus::prelude::*;
use crate::{APP_STORE_URL, Footer, Nav};

const LOGO_ASCII: &str = include_str!("../../assets/logo.txt");

fn fit_logo_to_viewport() {
    let Some(window) = web_sys::window() else { return };
    let Some(document) = window.document() else { return };
    let Some(logo) = document.query_selector(".logo").ok().flatten() else { return };

    // Reset to base size with animation paused so we measure natural width
    let _ = logo.set_attribute("style", "animation: none; font-size: 16px;");
    let _ = logo.scroll_width(); // force reflow

    let vw = window.inner_width().ok().and_then(|v| v.as_f64()).unwrap_or(1440.0);
    let width = logo.scroll_width() as f64;

    if width > 0.0 {
        let new_fs = 16.0 * vw / width;
        // Set computed font-size; dropping animation:none lets the CSS animation replay
        let _ = logo.set_attribute("style", &format!("font-size: {new_fs}px;"));
    }
}

#[component]
pub fn Home() -> Element {
    rsx! {
        Nav {}
        div { class: "hero",
            div {
                class: "logo",
                onmounted: move |_| {
                    spawn(async {
                        gloo_timers::future::TimeoutFuture::new(10).await;
                        fit_logo_to_viewport();
                    });
                },
                "{LOGO_ASCII}"
            }
            p { class: "tagline",
                "the mtg deck builder built for mobile. swipe right to add, left to skip."
            }
            a { href: APP_STORE_URL, class: "appstore-btn", "download on the app store" }
        }
        div { class: "page",
            div { class: "features-grid",
                div { class: "feature-card",
                    h3 { "swipe to build" }
                    p { "browse cards one at a time. right to add, left to skip. no clutter." }
                }
                div { class: "feature-card",
                    h3 { "deep filters" }
                    p { "filter by color, type, mana cost, oracle text, artist, set, and more." }
                }
                div { class: "feature-card",
                    h3 { "35k+ cards" }
                    p { "full scryfall card database, synced nightly. every card, every set." }
                }
                div { class: "feature-card",
                    h3 { "commander ready" }
                    p { "singleton mode, commander assignment, mana curve and color identity." }
                }
                div { class: "feature-card",
                    h3 { "import / export" }
                    p { "paste any decklist from moxfield or archidekt and it just works." }
                }
                div { class: "feature-card",
                    h3 { "your decks, synced" }
                    p { "account-based. your decks are always there across sessions." }
                }
            }
        }
        Footer {}
    }
}
