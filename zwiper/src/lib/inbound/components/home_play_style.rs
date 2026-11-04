//! Home's Colors panel: a bar per color with the card count across the
//! user's decks, and a Playstyle chip in its header that opens the full
//! screen.

use components::{Button, ButtonVariant, mana_var};
use dioxus::prelude::*;
use uuid::Uuid;
use zwipe_core::http::contracts::suggestions::HttpPlayStyleSummary;
use zwipe_ui::screens::home::{self, Bar, PlayStyle};

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

/// The fill of the colorless bar, which has no mana variable.
const COLORLESS_FILL: &str = "var(--text-muted)";

/// The panel between Decks and the featured flavor card, in the Decks
/// panel's style: Colors and its "?" centered in its header with the
/// Playstyle chip in the right corner, then one compact bar per color in W U B R G order
/// and colorless last, the pip at its left, the count at its end and the
/// largest count at full width. Before any deck has a card it shows one line
/// under the header instead; a failed fetch with nothing cached shows a muted
/// line. The header's Playstyle chip stays in both, so the screen is always
/// reachable from Home; while the fetch is out the whole panel is a ghost.
/// Home's model decides which.
#[component]
pub fn HomePlayStyle(play_style: PlayStyle, onplaystyle: EventHandler) -> Element {
    rsx! {
        div { class: "container-sm flex-col home-flavor home-colors content-enter-delayed",
            match play_style {
                PlayStyle::Bars(bars) => rsx! {
                    PanelHeader { onplaystyle }
                    ColorBars { bars }
                },
                PlayStyle::Empty => rsx! {
                    PanelHeader { onplaystyle }
                    ColorMapEmpty {}
                },
                PlayStyle::Failed => rsx! {
                    PanelHeader { onplaystyle }
                    div { class: "message-empty home-colors-line", {home::PLAY_STYLE_FAILED} }
                },
                PlayStyle::Loading => rsx! {
                    // Everything ghosts until the summary lands: the title
                    // and the "?" as bars their size in the middle, the chip
                    // as a bare chip in the corner, then the six bars with a
                    // pip and a count each.
                    div { class: "skeleton-card-header settings-list-header home-panel-header-centered",
                        div { class: "home-panel-header-title",
                            div { class: "skeleton-bar skeleton-section-label home-panel-skeleton-title" }
                            div { class: "skeleton-bar skeleton-chip skeleton-info-button" }
                        }
                        div { class: "home-panel-header-action",
                            div { class: "skeleton-bar skeleton-chip skeleton-chip-md" }
                        }
                    }
                    div { class: "color-bars",
                        for i in 0..6 {
                            div { key: "{i}", class: "color-bar-row",
                                div { class: "skeleton-bar color-bar-pip-ghost" }
                                div { class: "skeleton-bar color-bar-track" }
                                div { class: "skeleton-bar color-bar-count-ghost" }
                            }
                        }
                    }
                },
            }
        }
    }
}

/// The bars the model lists: the mana pip at the left, the track filled in
/// the color's mana variable (a muted neutral for colorless) to the bar's
/// share, and the count right-aligned at the end. A zero count draws its pip
/// and count over an empty track.
#[component]
fn ColorBars(bars: Vec<Bar>) -> Element {
    let rows = bars.into_iter().map(|bar| {
        let (pip, fill) = match bar.color {
            Some(color) => (
                color.to_short_name().to_lowercase(),
                format!("var({})", mana_var(color)),
            ),
            None => ("c".to_owned(), COLORLESS_FILL.to_owned()),
        };
        (pip, fill, bar.count, bar.pct)
    });
    rsx! {
        div { class: "color-bars",
            for (pip, fill, count, pct) in rows {
                div { key: "{pip}", class: "color-bar-row",
                    i { class: "ms ms-{pip} ms-cost" }
                    div { class: "color-bar-track",
                        if count > 0 {
                            div {
                                class: "color-bar-fill",
                                style: "width: {pct}%; background-color: {fill};",
                            }
                        }
                    }
                    span { class: "color-bar-count", "{count}" }
                }
            }
        }
    }
}

/// What a color map or the Colors bars show in their place before any deck
/// has a card, on Home and in Playstyle's Color map section.
#[component]
pub(crate) fn ColorMapEmpty() -> Element {
    rsx! {
        div { class: "message-empty home-colors-line", {home::NO_PLAY_STYLE} }
    }
}

/// The panel's header: Colors and its "?" centered as one unit, the way
/// the app header pairs its title and "?", the Playstyle chip alone in the
/// right corner.
#[component]
fn PanelHeader(onplaystyle: EventHandler) -> Element {
    rsx! {
        div { class: "settings-list-header home-panel-header-centered",
            div { class: "home-panel-header-title",
                span { class: "settings-list-title", {home::COLORS} }
                InfoButton { topic: HintTopic::HomeColors }
            }
            div { class: "home-panel-header-action",
                Button {
                    variant: ButtonVariant::Action,
                    onclick: move |_| onplaystyle.call(()),
                    {home::PLAYSTYLE}
                }
            }
        }
    }
}
