//! Home's Color map panel: the user's color territory map, with a Playstyle
//! button that opens the full screen.

use components::{ColorMap, ColorMapCounts, ColorMapGhost, seed_from_id};
use dioxus::prelude::*;
use uuid::Uuid;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::http::contracts::suggestions::HttpPlayStyleSummary;

use crate::inbound::components::auth::authed::use_authed;
use crate::inbound::components::hint_host::HintTopic;
use crate::inbound::components::info_button::InfoButton;
use crate::inbound::components::telemetry::vocabulary::Screen;
use crate::inbound::router::Router;
use crate::inbound::screens::deck::suggested_commanders::color_map_has_data;

/// The last summary fetched, kept above the router (as
/// `Signal<Option<CachedPlayStyle>>`) so a return to Home shows it at once
/// while the refetch is out.
#[derive(Debug, Clone, PartialEq)]
pub struct CachedPlayStyle {
    /// The user the summary belongs to; another user's copy is never shown.
    pub user_id: Uuid,
    /// What zerver answered.
    pub summary: HttpPlayStyleSummary,
}

/// Fetches the playstyle summary into the cache above the router and returns
/// this user's copy, if any, with whether the last fetch failed. A cached
/// copy shows while the refetch is out and is replaced only on success.
fn use_play_style_summary() -> (Option<HttpPlayStyleSummary>, Signal<bool>) {
    let session: Signal<Option<Session>> = use_context();
    let mut cache: Signal<Option<CachedPlayStyle>> = use_context();
    let mut failed = use_signal(|| false);
    let authed = use_authed(Screen::Home);

    let user_id = session.peek().as_ref().map(|s| s.user.id);
    use_effect(move || {
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

/// The panel under the hero, a sibling of the featured flavor card: the
/// title bar, then the user's [`ColorMap`], seeded by their id so it draws
/// the same shape as the Playstyle screen's Color map section, with a
/// Playstyle button at the end of its key. Before any deck has colored cards
/// it shows one line instead; a failed fetch with nothing cached shows a
/// muted line. Either line has the Playstyle button under it, so the screen
/// stays reachable from Home; while the fetch is out the whole panel is a
/// ghost.
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
                    let counts = summary
                        .color_map
                        .as_ref()
                        .map(ColorMapCounts::from)
                        .filter(color_map_has_data)
                        .filter(|_| summary.deck_count > 0);
                    rsx! {
                        PanelHeader {}
                        if let Some(counts) = counts {
                            ColorMap {
                                counts,
                                seed,
                                key_action: rsx! {
                                    PlaystyleButton {}
                                },
                            }
                        } else {
                            ColorMapEmpty {}
                            div { class: "home-panel-actions", PlaystyleButton {} }
                        }
                    }
                }
                None if failed() => rsx! {
                    PanelHeader {}
                    div { class: "message-empty", style: "padding: 1rem;",
                        "Could not load your playstyle"
                    }
                    div { class: "home-panel-actions", PlaystyleButton {} }
                },
                None => rsx! {
                    // Everything ghosts until the summary lands: the title as
                    // a bar, the "?" as a bare chip its size, then the map's
                    // ghost with the Playstyle button's at the key's end.
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

/// A [`ColorMap`] while its counts load: a [`ColorMapGhost`] of skeleton
/// tiles sweeping out from the middle in the map's own box, then a bare
/// chip for each of the key's five colors and a wider one for colorless,
/// and with `key_action` one more for the button at the key's end.
#[component]
pub(crate) fn ColorMapSkeleton(#[props(default)] key_action: bool) -> Element {
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

/// What a color map shows in its place before any deck has colored cards, on
/// Home and in Playstyle's Color map section.
#[component]
pub(crate) fn ColorMapEmpty() -> Element {
    rsx! {
        div { class: "message-empty", style: "padding: 1rem;", "Build a deck to see your playstyle" }
    }
}

/// The panel's title bar: Color map centered and its "?" at the right end,
/// where the app's other headers keep theirs.
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

/// Opens the Playstyle screen, in the card details bar's button grammar.
#[component]
fn PlaystyleButton() -> Element {
    let navigator = use_navigator();
    rsx! {
        button {
            class: "card-action-btn",
            onclick: move |_| {
                navigator.push(Router::SuggestedCommanders {});
            },
            "Playstyle"
        }
    }
}
