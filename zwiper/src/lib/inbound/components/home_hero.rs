//! The mark in a panel with the three public counters, on home and the auth
//! screens.

use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use std::time::{Duration, Instant};
use zwipe_client::ZwipeClient;
use zwipe_components::{CountUp, Decode, Panel};
use zwipe_core::{domain::logo, http::contracts::metrics::HttpPublicMetrics};

/// How long a fetched set of counters serves before the next screen asks
/// zerver again. Hopping between home and the auth screens reuses it.
const COUNTS_FRESH: Duration = Duration::from_secs(60);

/// The counters as last fetched, kept above the router (as
/// `Signal<Option<CachedCounts>>`) so every screen with the panel shares one
/// copy and one fetch per minute.
#[derive(Clone)]
pub struct CachedCounts {
    /// What zerver answered.
    pub metrics: HttpPublicMetrics,
    /// When it answered.
    pub fetched_at: Instant,
}

impl CachedCounts {
    /// Older than `COUNTS_FRESH`, so the next screen asks again.
    pub fn is_stale(&self) -> bool {
        self.fetched_at.elapsed() >= COUNTS_FRESH
    }
}

/// The mark above the cards swiped, searches run and decks created across
/// everyone, with chips saying where they come from. Unauthenticated; the
/// figures roll until they arrive, and a failed ask toasts the way every other
/// failed ask does.
#[component]
pub fn HomeHero() -> Element {
    let client: Signal<ZwipeClient> = use_context();
    let toast = use_toast();
    let mut cache: Signal<Option<CachedCounts>> = use_context();
    // A fresh copy is reused; a stale one keeps showing while the refetch is
    // out and is only replaced on success.
    use_effect(move || {
        if cache.peek().as_ref().is_some_and(|c| !c.is_stale()) {
            return;
        }
        spawn(async move {
            match client().public_metrics().await {
                Ok(metrics) => cache.set(Some(CachedCounts {
                    metrics,
                    fetched_at: Instant::now(),
                })),
                Err(e) => toast.error(e.to_string(), ToastOptions::default()),
            }
        });
    });

    let value = cache.read();
    let figures = value.as_ref().map(|c| &c.metrics);
    let count = |n: i64| u64::try_from(n).unwrap_or(0);
    rsx! {
        div { class: "home-hero",
            Panel {
                div { class: "hero-head",
                    div { class: "logo", Decode { text: logo::ZWIPE } }
                    section { class: "stats-strip",
                        div { class: "stat",
                            span { class: "stat-num", CountUp { value: figures.map(|m| count(m.decks_created)), compact: true } }
                            span { class: "stat-label", "Decks created" }
                        }
                        div { class: "stat",
                            span { class: "stat-num", CountUp { value: figures.map(|m| count(m.searches)), compact: true } }
                            span { class: "stat-label", "Searches run" }
                        }
                        div { class: "stat",
                            span { class: "stat-num", CountUp { value: figures.map(|m| count(m.cards_swiped)), compact: true } }
                            span { class: "stat-label", "Cards swiped" }
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
