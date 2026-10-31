//! The mark over everybody's three counters, on the public screens.

use chrono::Utc;
use components::{Decode, Panel, StatsStrip};
use dioxus::prelude::*;
use zwipe_client::ZwipeClient;
use zwipe_core::domain::logo;

use crate::domain::counts::{self, CachedCounts};
use crate::outbound::counts_store::CountsStore;

/// The app's `HomeHero`: the decoding mark, then decks created, searches run
/// and cards swiped across everyone, tagged `Everybody` and, once fetched
/// during this visit, `Live`.
///
/// The counters come from the root's shared copy. A mount refetches when that
/// copy is missing, stored by an earlier visit, or a minute old, and keeps
/// showing it until the fetch lands. A failed fetch is only logged. Every
/// mount replays the decode and the roll.
#[component]
pub fn HomeHero() -> Element {
    let client: Signal<ZwipeClient> = use_context();
    let mut cache: Signal<Option<CachedCounts>> = use_context();
    use_hook(move || {
        if !counts::needs_fetch(cache.peek().as_ref(), Utc::now()) {
            return;
        }
        spawn(async move {
            let http = client.peek().clone();
            match http.public_metrics().await {
                Ok(metrics) => {
                    CountsStore::browser().save(&metrics);
                    cache.set(Some(CachedCounts::fetched(metrics, Utc::now())));
                }
                Err(e) => tracing::warn!("public counters not refreshed: {e}"),
            }
        });
    });
    let cached = cache.read();
    let figures = counts::public_figures(cached.as_ref().map(|c| &c.metrics));
    let live = cached.as_ref().is_some_and(CachedCounts::is_live);

    rsx! {
        div { class: "home-hero",
            Panel {
                div { class: "hero-head",
                    div { class: "logo", Decode { text: logo::ZWIPE } }
                    StatsStrip {
                        figures,
                        compact: true,
                        source: rsx! {
                            span { class: "stat-chip", "Everybody" }
                            if live {
                                span { class: "stat-chip", "Live" }
                            }
                        },
                    }
                }
            }
        }
    }
}
