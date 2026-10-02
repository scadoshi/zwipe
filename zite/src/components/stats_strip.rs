//! Live aggregate stats strip surfaced on the marketing site.
//!
//! Fetched during SSR. CF caches the
//! API response at the edge (~2h TTL), GH Pages caches the rendered HTML,
//! so cost-per-pageview is near zero. On error the strip hides itself;
//! don't break the marketing page on a metrics outage.

use crate::api;
use dioxus::prelude::*;
use zwipe_components::CountUp;
use zwipe_core::http::contracts::metrics::HttpPublicMetrics;

#[component]
pub fn StatsStrip() -> Element {
    let stats: Resource<Option<HttpPublicMetrics>> =
        use_resource(|| async { api::client().public_metrics().await.ok() });

    let value = stats.read();
    let Some(Some(s)) = &*value else {
        return rsx! {};
    };

    rsx! {
        section { class: "stats-strip",
            div { class: "stat",
                span { class: "stat-num", CountUp { value: u64::try_from(s.cards_swiped).unwrap_or(0) } }
                span { class: "stat-label", "Cards swiped" }
            }
            div { class: "stat",
                span { class: "stat-num", CountUp { value: u64::try_from(s.searches).unwrap_or(0) } }
                span { class: "stat-label", "Searches run" }
            }
            div { class: "stat",
                span { class: "stat-num", CountUp { value: u64::try_from(s.decks_created).unwrap_or(0) } }
                span { class: "stat-label", "Decks created" }
            }
        }
    }
}
