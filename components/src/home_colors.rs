//! Home's Colors panel: the shared [`ColorBars`], a bar per color with the
//! card count across the user's decks, and a Playstyle chip in its header
//! that opens the full screen.

use dioxus::prelude::*;
use orangui::{Button, ButtonVariant, InfoButton};
use zwipe_ui::screens::home::{self, PlayStyle};

use crate::color_bars::{ColorBars, ColorBarsGhost};

/// The panel between Decks and the featured flavor card, in the Decks
/// panel's style: Colors and its "?" centered in its header with the
/// Playstyle chip in the right corner, then one compact bar per color in W U B R G order
/// and colorless last, the pip at its left, the count at its end and the
/// largest count at full width. Before any deck has a card it shows one line
/// under the header instead; a failed fetch with nothing cached shows a muted
/// line. The header's Playstyle chip stays in both, so the screen is always
/// reachable from Home; while the fetch is out the whole panel is a ghost.
/// Home's model decides which.
///
/// `onplaystyle` is the Playstyle chip and `oninfo` the header's "?", which
/// the host answers with its Colors hint.
#[component]
pub fn HomeColors(
    play_style: PlayStyle,
    onplaystyle: EventHandler,
    oninfo: EventHandler,
) -> Element {
    rsx! {
        div { class: "container-sm flex-col home-flavor home-colors content-enter-delayed",
            match play_style {
                PlayStyle::Bars(bars) => rsx! {
                    PanelHeader { onplaystyle, oninfo }
                    ColorBars { bars }
                },
                PlayStyle::Empty => rsx! {
                    PanelHeader { onplaystyle, oninfo }
                    ColorsEmpty {}
                },
                PlayStyle::Failed => rsx! {
                    PanelHeader { onplaystyle, oninfo }
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
                    ColorBarsGhost {}
                },
            }
        }
    }
}

/// What the Colors bars show in their place before any deck has a card, on
/// Home and in Playstyle's Colors section.
#[component]
pub fn ColorsEmpty() -> Element {
    rsx! {
        div { class: "message-empty home-colors-line", {home::NO_PLAY_STYLE} }
    }
}

/// The panel's header: Colors and its "?" centered as one unit, the way
/// the app header pairs its title and "?", the Playstyle chip alone in the
/// right corner.
#[component]
fn PanelHeader(onplaystyle: EventHandler, oninfo: EventHandler) -> Element {
    rsx! {
        div { class: "settings-list-header home-panel-header-centered",
            div { class: "home-panel-header-title",
                span { class: "settings-list-title", {home::COLORS} }
                InfoButton { onclick: move |_| oninfo.call(()) }
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
