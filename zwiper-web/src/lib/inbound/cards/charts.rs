//! The chart and color map pieces Home, Activity and Playstyle share.

use components::ColorMapGhost;
use dioxus::prelude::*;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::http::contracts::suggestions::HttpColorMap;
use zwipe_ui::activity::Bar;
use zwipe_ui::color_map::{self, seed_from_id};
use zwipe_ui::playstyle::color_map_has_data;

/// zwipe-ui's chart bars as the kit's chart draws them.
pub fn chart_bars(bars: Vec<Bar>) -> Vec<components::Bar> {
    bars.into_iter()
        .map(|bar| components::Bar {
            key: bar.key,
            label: bar.label,
            count: bar.count,
            pct: bar.pct,
            revealable: bar.revealable,
        })
        .collect()
}

/// The color map's seed for the signed-in user, so Home and Playstyle draw
/// the same shape; 0 without a session.
pub fn user_seed(session: Signal<Option<Session>>) -> u64 {
    session
        .peek()
        .as_ref()
        .map_or(0, |s| seed_from_id(s.user.id.as_u128()))
}

/// The kit's counts for `map`, when it has a colored or colorless card on
/// it (zwipe-ui's rule).
pub fn map_counts(map: &HttpColorMap) -> Option<components::ColorMapCounts> {
    color_map_has_data(&color_map::ColorMapCounts::from(map))
        .then(|| components::ColorMapCounts::from(map))
}

/// A color map while its counts load: the kit's ghost map, then a bare chip
/// for each of the key's five colors, a wider one for colorless, and with
/// `key_action` one more for the button at the key's end.
#[component]
pub fn ColorMapSkeleton(#[props(default)] key_action: bool) -> Element {
    rsx! {
        div { class: "color-map",
            ColorMapGhost {}
            div { class: "color-map-key",
                for i in 0..5 {
                    div { key: "{i}", class: "skeleton-bar skeleton-chip skeleton-chip-md" }
                }
                div { class: "skeleton-bar skeleton-chip skeleton-chip-lg" }
                if key_action {
                    div { class: "skeleton-bar skeleton-chip skeleton-chip-lg" }
                }
            }
        }
    }
}

/// The line in a color map's place before any deck has colored cards, on
/// Home and in Playstyle's Color map section.
#[component]
pub fn ColorMapEmpty() -> Element {
    rsx! {
        div { class: "message-empty", style: "padding: 1rem;", "Build a deck to see your playstyle" }
    }
}
