//! Home's Color map panel: the user's color territory map, with a Playstyle
//! button that opens the full screen.

use components::{ColorMap, ColorMapCounts, ColorMapGhost};
use dioxus::prelude::*;
use uuid::Uuid;
use zwipe_core::http::contracts::suggestions::HttpPlayStyleSummary;
use zwipe_ui::screens::home::{self, PlayStyle};

use crate::inbound::components::hint_host::HintTopic;
use crate::inbound::components::info_button::InfoButton;

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

/// The panel under the hero, a sibling of the featured flavor card: the
/// title bar, then the user's [`ColorMap`], seeded by their id so it draws
/// the same shape as the Playstyle screen's Color map section, with a
/// Playstyle button at the end of its key. Before any deck has colored cards
/// it shows one line instead; a failed fetch with nothing cached shows a
/// muted line. Either line has the Playstyle button under it, so the screen
/// stays reachable from Home; while the fetch is out the whole panel is a
/// ghost. Home's model decides which.
#[component]
pub fn HomePlayStyle(play_style: PlayStyle, seed: u64, onplaystyle: EventHandler) -> Element {
    rsx! {
        div { class: "container-sm text-center flex-col home-flavor content-enter-delayed",
            match play_style {
                PlayStyle::Map(_) | PlayStyle::Empty => {
                    let counts = match play_style {
                        PlayStyle::Map(counts) => Some(ColorMapCounts {
                            cards: counts.cards,
                            colorless: counts.colorless,
                        }),
                        PlayStyle::Empty | PlayStyle::Failed | PlayStyle::Loading => None,
                    };
                    rsx! {
                        PanelHeader {}
                        if let Some(counts) = counts {
                            ColorMap {
                                counts,
                                seed,
                                key_action: rsx! {
                                    PlaystyleButton { onclick: onplaystyle }
                                },
                            }
                        } else {
                            ColorMapEmpty {}
                            div { class: "home-panel-actions", PlaystyleButton { onclick: onplaystyle } }
                        }
                    }
                }
                PlayStyle::Failed => rsx! {
                    PanelHeader {}
                    div { class: "message-empty", style: "padding: 1rem;",
                        {home::PLAY_STYLE_FAILED}
                    }
                    div { class: "home-panel-actions", PlaystyleButton { onclick: onplaystyle } }
                },
                PlayStyle::Loading => rsx! {
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
/// tiles sweeping out from the middle in the map's own box, then under its
/// rule a bare chip for each of the key's five colors and a wider one for
/// colorless, and with `key_action` one more for the button at the key's
/// end.
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
        div { class: "message-empty", style: "padding: 1rem;", {home::NO_PLAY_STYLE} }
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
            span { class: "settings-list-title", {home::COLOR_MAP} }
        }
    }
}

/// Opens the Playstyle screen, in the card details bar's button grammar.
#[component]
fn PlaystyleButton(onclick: EventHandler) -> Element {
    rsx! {
        button {
            class: "card-action-btn",
            onclick: move |_| onclick.call(()),
            {home::PLAYSTYLE}
        }
    }
}
