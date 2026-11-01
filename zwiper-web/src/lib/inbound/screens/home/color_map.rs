//! Home's color map panel: the user's color map from every deck. The nav's
//! `Playstyle` opens the full screen.

use components::ColorMap;
use dioxus::prelude::*;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::http::contracts::suggestions::{HttpColorMap, HttpPlayStyleSummary};

use crate::inbound::caches::CachedPlayStyle;
use crate::inbound::cards::charts::{ColorMapEmpty, ColorMapSkeleton, map_counts, user_seed};
use crate::inbound::chrome::authed::use_authed;
use crate::inbound::chrome::hint_topic::{HintTopic, InfoButton};

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
    map_counts(map?).filter(|_| deck_count > 0)
}

/// The panel: its title with a `?`, then the user's color map seeded by
/// their id (the same shape as Playstyle's). Before any deck has colored
/// cards, one line; a failed fetch with nothing cached, a muted line. While
/// the fetch is out the whole panel ghosts.
#[component]
pub fn HomePlayStyle() -> Element {
    let (summary, failed) = use_play_style_summary();
    let seed = user_seed(use_context());
    rsx! {
        div { class: "container-sm text-center flex-col home-flavor content-enter-delayed",
            match summary {
                Some(summary) => {
                    let counts = drawable(summary.color_map.as_ref(), summary.deck_count);
                    rsx! {
                        PanelHeader {}
                        if let Some(counts) = counts {
                            ColorMap { counts, seed }
                        } else {
                            ColorMapEmpty {}
                        }
                    }
                }
                None if failed() => rsx! {
                    PanelHeader {}
                    div { class: "message-empty", style: "padding: 1rem;", "Could not load your playstyle" }
                },
                None => rsx! {
                    div { class: "skeleton-card-header home-panel-header", style: "display: flex;",
                        span { class: "home-panel-hint",
                            div { class: "skeleton-bar skeleton-chip skeleton-info-button" }
                        }
                        div { class: "skeleton-bar skeleton-section-label", style: "width: 30%; margin-bottom: 0;" }
                    }
                    ColorMapSkeleton {}
                },
            }
        }
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
