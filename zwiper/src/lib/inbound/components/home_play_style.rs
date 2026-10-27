//! Home's Playstyle panel: the user's color territory map, with a button that
//! opens the full screen.

use components::{Chip, ColorMap, ColorMapCounts, seed_from_id};
use dioxus::prelude::*;
use uuid::Uuid;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::http::contracts::suggestions::HttpPlayStyleSummary;

use crate::inbound::components::auth::authed::use_authed;
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
/// title bar with an Open chip, then the user's [`ColorMap`], seeded by their
/// id so it draws the same shape as the Playstyle screen's Colors section.
/// Before any deck has colored cards it shows one line instead; a failed
/// fetch with nothing cached shows a muted line. Once the fetch settles the
/// Open chip is always there, so the screen stays reachable from Home; while
/// it is out the whole panel is a ghost.
#[component]
pub fn HomePlayStyle() -> Element {
    let navigator = use_navigator();
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
                        PanelHeader { onclick: move |_| { navigator.push(Router::SuggestedCommanders {}); } }
                        if let Some(counts) = counts {
                            ColorMap { counts, seed }
                        } else {
                            div { class: "message-empty", style: "padding: 1rem;",
                                "Build a deck to see your playstyle"
                            }
                        }
                    }
                }
                None if failed() => rsx! {
                    PanelHeader { onclick: move |_| { navigator.push(Router::SuggestedCommanders {}); } }
                    div { class: "message-empty", style: "padding: 1rem;",
                        "Could not load your playstyle"
                    }
                },
                None => rsx! {
                    // Everything ghosts until the summary lands: the title as
                    // a bar, Open as a bare chip, then the map's ghost.
                    div { class: "skeleton-card-header home-panel-header", style: "display: flex;",
                        div { class: "skeleton-bar skeleton-section-label", style: "width: 30%; margin-bottom: 0;" }
                        span { class: "home-panel-open",
                            div { class: "skeleton-bar skeleton-chip skeleton-chip-sm" }
                        }
                    }
                    ColorMapSkeleton {}
                },
            }
        }
    }
}

/// A [`ColorMap`] while its counts load: one block the size of the map's
/// full viewport, with no tiles, then bare chips for the key.
#[component]
pub(crate) fn ColorMapSkeleton() -> Element {
    rsx! {
        div { class: "color-map",
            div { class: "skeleton-bar color-map-ghost" }
            div { class: "color-map-key",
                for i in 0..3 {
                    div { key: "{i}", class: "skeleton-bar skeleton-chip skeleton-chip-sm" }
                }
            }
        }
    }
}

/// The panel's title bar: Playstyle centered, Open as a chip at its right
/// end, the way the deck view's Budget header carries its currency chips.
#[component]
fn PanelHeader(onclick: EventHandler<MouseEvent>) -> Element {
    rsx! {
        div { class: "settings-list-header home-panel-header",
            span { class: "settings-list-title", "Playstyle" }
            span { class: "home-panel-open",
                Chip { selected: false, onclick: move |e| onclick.call(e), "Open" }
            }
        }
    }
}
