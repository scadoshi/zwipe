//! The three public counters in the hero, rounded as the app shows them
//! (31.8k, 264.4k).
//!
//! `metrics.json` is zerver's answer as of the last deploy, refreshed by the
//! deploy workflow before each build and baked into the prerender, so the page
//! carries real numbers before any script runs. After load the browser asks
//! zerver once and swaps the answer in; the chip under the strip says which
//! one is showing: the deploy's date, or `Live` while the answer's copy is
//! new and "as of" its day when the edge cache served an older one.

use std::sync::LazyLock;

use chrono::{NaiveDate, Utc};
use dioxus::prelude::*;
use serde::Deserialize;
use zwipe_client::call::Dated;
use zwipe_core::http::contracts::metrics::HttpPublicMetrics;
use zwipe_ui::home::{self, CountsTag};

use crate::api;

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
    let stats: Resource<Option<Dated<HttpPublicMetrics>>> = use_resource(|| async {
        if !cfg!(target_arch = "wasm32") {
            return None;
        }
        api::client().fetch_public_metrics_dated().await.ok()
    });

    let value = stats.read();
    let fetched = value.as_ref().and_then(|stats| stats.as_ref());
    let live = fetched.map(|stats| &stats.value);
    let baked = BAKED.as_ref();
    // The live answer, else the baked one; with neither the figures roll.
    let figures = live.or_else(|| baked.map(|baked| &baked.metrics));
    // The deploy's day, from the baked copy's RFC 3339 stamp; a stamp that
    // does not parse shows as it is.
    let as_of = baked.map(|baked| {
        let stamp = &baked.fetched_at;
        stamp
            .get(..10)
            .and_then(|day| NaiveDate::parse_from_str(day, "%Y-%m-%d").ok())
            .map_or_else(|| format!("as of {stamp}"), home::as_of)
    });

    let count =
        |pick: fn(&HttpPublicMetrics) -> i64| figures.map(|s| u64::try_from(pick(s)).unwrap_or(0));
    // Keyed on its text, so the live chip arrives with the ease rather than
    // the as-of chip changing its words in place.
    let label = match (fetched, as_of) {
        (Some(stats), _) => Some(CountsTag::at(stats.made_at, Utc::now()).label()),
        (None, Some(as_of)) => Some(as_of),
        (None, None) => None,
    };

    rsx! {
        components::StatsStrip {
            figures: vec![
                (count(|s| s.decks_created), "Decks created"),
                (count(|s| s.searches), "Searches run"),
                (count(|s| s.cards_swiped), "Cards swiped"),
            ],
            compact: true,
            // Where the numbers come from, as chips under the strip.
            source: rsx! {
                if let Some(label) = label {
                    span { key: "{label}", class: "tag chip-in", "{label}" }
                }
                span { class: "tag", "Everybody" }
            },
        }
    }
}
