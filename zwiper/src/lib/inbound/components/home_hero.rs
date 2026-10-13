//! The mark in a panel with the three public counters, on home and the auth
//! screens.

use crate::outbound::counts_store::PersistCounts;
use components::{Decode, Panel, StatsStrip};
use dioxus::prelude::*;
use std::time::{Duration, Instant};
use zwipe_client::ZwipeClient;
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
    /// When it answered this launch; `None` for a copy restored from the last
    /// launch, which serves until a fetch replaces it.
    pub fetched_at: Option<Instant>,
}

impl CachedCounts {
    /// The copy saved by the last launch, if there is one.
    pub fn restored() -> Option<Self> {
        HttpPublicMetrics::infallible_load().map(|metrics| Self {
            metrics,
            fetched_at: None,
        })
    }

    /// Older than `COUNTS_FRESH`, or restored, so the next screen asks again.
    pub fn is_stale(&self) -> bool {
        self.fetched_at
            .is_none_or(|fetched_at| fetched_at.elapsed() >= COUNTS_FRESH)
    }
}

/// The mark above the cards swiped, searches run and decks created across
/// everyone, with chips saying where they come from. Unauthenticated; the
/// figures roll until they arrive. A failed ask keeps the copy on screen,
/// restored from the last launch if need be, and is only logged: these
/// numbers are decoration, not something the user asked for.
#[component]
pub fn HomeHero() -> Element {
    let client: Signal<ZwipeClient> = use_context();
    let mut cache: Signal<Option<CachedCounts>> = use_context();
    // A fresh copy is reused; a stale one keeps showing while the refetch is
    // out and is only replaced on success.
    use_effect(move || {
        if cache.peek().as_ref().is_some_and(|c| !c.is_stale()) {
            return;
        }
        spawn(async move {
            match client().public_metrics().await {
                Ok(metrics) => {
                    metrics.infallible_save();
                    cache.set(Some(CachedCounts {
                        metrics,
                        fetched_at: Some(Instant::now()),
                    }));
                }
                Err(e) => tracing::warn!("public counters not refreshed: {e}"),
            }
        });
    });

    let value = cache.read();
    let metrics = value.as_ref().map(|c| &c.metrics);
    let count = |n: Option<i64>| n.map(|n| u64::try_from(n).unwrap_or(0));
    let figures = vec![
        (count(metrics.map(|m| m.decks_created)), "Decks created"),
        (count(metrics.map(|m| m.searches)), "Searches run"),
        (count(metrics.map(|m| m.cards_swiped)), "Cards swiped"),
    ];
    // "live" only for an answer from this launch, not a restored copy.
    let live = value.as_ref().is_some_and(|c| c.fetched_at.is_some());
    rsx! {
        div { class: "home-hero",
            Panel {
                div { class: "hero-head",
                    div { class: "logo", Decode { text: logo::ZWIPE } }
                    StatsStrip {
                        figures,
                        compact: true,
                        // Where the numbers come from, as chips under the strip.
                        source: rsx! {
                            span { class: "stat-chip", "all users" }
                            if live {
                                span { class: "stat-chip", "live" }
                            }
                        },
                    }
                }
            }
        }
    }
}
