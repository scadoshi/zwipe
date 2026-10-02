//! Live aggregate stats strip surfaced on the marketing site.
//!
//! Fetched during SSR. CF caches the
//! API response at the edge (~2h TTL), GH Pages caches the rendered HTML,
//! so cost-per-pageview is near zero. On error the strip hides itself;
//! don't break the marketing page on a metrics outage.

use crate::{api, components::CountUp};
use dioxus::prelude::*;
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
        hr { class: "hero-rule" }
        section { class: "stats-strip",
            div { class: "stat",
                span { class: "stat-num", CountUp { value: s.cards_swiped } }
                span { class: "stat-label", "Cards swiped" }
            }
            div { class: "stat",
                span { class: "stat-num", CountUp { value: s.searches } }
                span { class: "stat-label", "Searches run" }
            }
            div { class: "stat",
                span { class: "stat-num", CountUp { value: s.decks_created } }
                span { class: "stat-label", "Decks created" }
            }
        }
    }
}
