//! The three public counters in the hero.
//!
//! `metrics.json` is zerver's answer as of the last deploy, refreshed by the
//! deploy workflow before each build and baked into the prerender, so the page
//! carries real numbers before any script runs. After load the browser asks
//! zerver once and swaps the live answer in; the chip under the strip says
//! which one is showing.

use crate::{Route, api};
use dioxus::prelude::*;
use serde::Deserialize;
use std::sync::LazyLock;
use zwipe_components::CountUp;
use zwipe_core::http::contracts::metrics::HttpPublicMetrics;

/// What the deploy workflow writes: when it asked, and what zerver said.
#[derive(Deserialize)]
struct Baked {
    fetched_at: String,
    metrics: HttpPublicMetrics,
}

/// The last deploy's answer. `None` only if the file does not parse, and the
/// figures then roll until the live answer arrives.
static BAKED: LazyLock<Option<Baked>> =
    LazyLock::new(|| serde_json::from_str(include_str!("../metrics.json")).ok());

#[component]
pub fn StatsStrip() -> Element {
    // Only the browser asks; the prerender shows the baked copy and says so.
    let stats: Resource<Option<HttpPublicMetrics>> = use_resource(|| async {
        if !cfg!(target_arch = "wasm32") {
            return None;
        }
        api::client().public_metrics().await.ok()
    });

    let value = stats.read();
    let live = value.as_ref().and_then(|stats| stats.as_ref());
    let baked = BAKED.as_ref();
    // The live answer, else the baked one; with neither the figures roll.
    let figures = live.or_else(|| baked.map(|baked| &baked.metrics));
    let as_of = baked.map(|baked| baked.fetched_at.get(..10).unwrap_or(&baked.fetched_at));

    rsx! {
        div { class: "hero-figures",
            section { class: "stats-strip",
                div { class: "stat",
                    span { class: "stat-num", CountUp { value: figures.map(|s| u64::try_from(s.cards_swiped).unwrap_or(0)) } }
                    span { class: "stat-label", "Cards swiped" }
                }
                div { class: "stat",
                    span { class: "stat-num", CountUp { value: figures.map(|s| u64::try_from(s.searches).unwrap_or(0)) } }
                    span { class: "stat-label", "Searches run" }
                }
                div { class: "stat",
                    span { class: "stat-num", CountUp { value: figures.map(|s| u64::try_from(s.decks_created).unwrap_or(0)) } }
                    span { class: "stat-label", "Decks created" }
                }
            }
            // Where the numbers come from, as chips under the strip.
            div { class: "tag-row stats-source",
                Link { class: "tag", to: Route::About {}, "counted by zerver" }
                // Keyed on its text, so the live chip arrives with the ease
                // rather than the as-of chip changing its words in place.
                {
                    let label = match (live.is_some(), as_of) {
                        (true, _) => Some("live".to_string()),
                        (false, Some(day)) => Some(format!("as of {day}")),
                        (false, None) => None,
                    };
                    rsx! {
                        if let Some(label) = label {
                            span { key: "{label}", class: "tag tag-swap", "{label}" }
                        }
                    }
                }
            }
        }
    }
}
