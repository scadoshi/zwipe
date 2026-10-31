//! Home's color map panel: the user's color map from every deck, with a
//! `Playstyle` button that opens the full screen.

use components::{ColorMap, ColorMapGhost};
use dioxus::prelude::*;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::http::contracts::suggestions::{HttpColorMap, HttpPlayStyleSummary};
use zwipe_ui::color_map::seed_from_id;
use zwipe_ui::playstyle::color_map_has_data;

use crate::inbound::caches::CachedPlayStyle;
use crate::inbound::chrome::authed::use_authed;
use crate::inbound::chrome::hint_topic::{HintTopic, InfoButton};
use crate::inbound::router::Route;

/// The playstyle summary from the root's copy, refetched quietly on every
/// mount; with whether the last fetch failed. The copy shows while the
/// refetch is out and is replaced only on success.
fn use_play_style_summary() -> (Option<HttpPlayStyleSummary>, Signal<bool>) {
    let session: Signal<Option<Session>> = use_context();
    let mut cache: Signal<Option<CachedPlayStyle>> = use_context();
    let mut failed = use_signal(|| false);
    let authed = use_authed();
    let user_id = session.peek().as_ref().map(|s| s.user.id);
    use_hook(move || {
        let Some(user_id) = user_id else {
            return;
        };
        spawn(async move {
            match authed
                .run_quiet("play_style_summary", |c, s| async move {
                    c.play_style_summary(&s).await
                })
                .await
            {
                Some(summary) => cache.set(Some(CachedPlayStyle { user_id, summary })),
                None => failed.set(true),
            }
        });
    });
    let summary = cache
        .read()
        .as_ref()
        .filter(|cached| Some(cached.user_id) == user_id)
        .map(|cached| cached.summary.clone());
    (summary, failed)
}

/// The kit's counts for `map`, when it has anything to draw (zwipe-ui's
/// rule) and the user has a deck.
fn drawable(map: Option<&HttpColorMap>, deck_count: u32) -> Option<components::ColorMapCounts> {
    let map = map?;
    let has_data = color_map_has_data(&zwipe_ui::color_map::ColorMapCounts::from(map));
    (has_data && deck_count > 0).then(|| components::ColorMapCounts::from(map))
}

/// The panel: its title with a `?`, then the user's color map seeded by
/// their id (the same shape as Playstyle's) with `Playstyle` at the end of
/// its key. Before any deck has colored cards, one line; a failed fetch
/// with nothing cached, a muted line; either with `Playstyle` under it.
/// While the fetch is out the whole panel ghosts.
#[component]
pub fn HomePlayStyle() -> Element {
    let (summary, failed) = use_play_style_summary();
    let session: Signal<Option<Session>> = use_context();
    let seed = session
        .peek()
        .as_ref()
        .map_or(0, |s| seed_from_id(s.user.id.as_u128()));
    rsx! {
        div { class: "container-sm text-center flex-col home-flavor content-enter-delayed",
            match summary {
                Some(summary) => {
                    let counts = drawable(summary.color_map.as_ref(), summary.deck_count);
                    rsx! {
                        PanelHeader {}
                        if let Some(counts) = counts {
                            ColorMap { counts, seed, key_action: rsx! { PlaystyleButton {} } }
                        } else {
                            ColorMapEmpty {}
                            div { class: "home-panel-actions", PlaystyleButton {} }
                        }
                    }
                }
                None if failed() => rsx! {
                    PanelHeader {}
                    div { class: "message-empty", style: "padding: 1rem;", "Could not load your playstyle" }
                    div { class: "home-panel-actions", PlaystyleButton {} }
                },
                None => rsx! {
                    div { class: "skeleton-card-header home-panel-header", style: "display: flex;",
                        span { class: "home-panel-hint",
                            div { class: "skeleton-bar skeleton-chip skeleton-info-button" }
                        }
                        div { class: "skeleton-bar skeleton-section-label", style: "width: 30%; margin-bottom: 0;" }
                    }
                    ColorMapSkeleton { key_action: true }
                },
            }
        }
    }
}

/// A color map while its counts load: the kit's ghost map, then a bare chip
/// for each of the key's five colors, a wider one for colorless, and with
/// `key_action` one more for the button.
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

/// The line in a color map's place before any deck has colored cards.
#[component]
pub fn ColorMapEmpty() -> Element {
    rsx! {
        div { class: "message-empty", style: "padding: 1rem;", "Build a deck to see your playstyle" }
    }
}

/// The panel's title bar: `Color map` centered, its `?` at the right end.
#[component]
fn PanelHeader() -> Element {
    rsx! {
        div { class: "settings-list-header home-panel-header",
            span { class: "home-panel-hint",
                InfoButton { topic: HintTopic::ColorMap }
            }
            span { class: "settings-list-title", "Color map" }
        }
    }
}

/// Opens Playstyle.
#[component]
fn PlaystyleButton() -> Element {
    let navigator = use_navigator();
    rsx! {
        button {
            class: "card-action-btn",
            onclick: move |_| {
                navigator.push(Route::Playstyle {});
            },
            "Playstyle"
        }
    }
}
