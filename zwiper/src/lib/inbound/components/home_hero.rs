//! The mark in a panel with the three public counters, on home and the auth
//! screens.

use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use zwipe_client::ZwipeClient;
use zwipe_components::{CountUp, Decode, Panel};
use zwipe_core::{domain::logo, http::contracts::metrics::HttpPublicMetrics};

/// The mark above the cards swiped, searches run and decks created across
/// everyone, with chips saying where they come from. Unauthenticated; the
/// figures roll until they arrive, and a failed ask toasts the way every other
/// failed ask does.
#[component]
pub fn HomeHero() -> Element {
    let client: Signal<ZwipeClient> = use_context();
    let toast = use_toast();
    let mut metrics: Signal<Option<HttpPublicMetrics>> = use_signal(|| None);
    use_effect(move || {
        spawn(async move {
            match client().public_metrics().await {
                Ok(m) => metrics.set(Some(m)),
                Err(e) => toast.error(e.to_string(), ToastOptions::default()),
            }
        });
    });

    let value = metrics.read();
    let figures = value.as_ref();
    let count = |n: i64| u64::try_from(n).unwrap_or(0);
    rsx! {
        div { class: "home-hero",
            Panel {
                div { class: "hero-head",
                    div { class: "logo", Decode { text: logo::ZWIPE } }
                    section { class: "stats-strip",
                        div { class: "stat",
                            span { class: "stat-num", CountUp { value: figures.map(|m| count(m.cards_swiped)) } }
                            span { class: "stat-label", "Cards swiped" }
                        }
                        div { class: "stat",
                            span { class: "stat-num", CountUp { value: figures.map(|m| count(m.searches)) } }
                            span { class: "stat-label", "Searches run" }
                        }
                        div { class: "stat",
                            span { class: "stat-num", CountUp { value: figures.map(|m| count(m.decks_created)) } }
                            span { class: "stat-label", "Decks created" }
                        }
                    }
                    // Where the numbers come from, as chips under the strip.
                    div { class: "home-source",
                        span { class: "stat-chip", "counted by zerver" }
                        if figures.is_some() {
                            span { class: "stat-chip", "live" }
                        }
                    }
                }
            }
        }
    }
}
