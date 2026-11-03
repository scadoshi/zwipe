//! Home's Decks panel: the user's deck count as a tile, with an Open decks
//! button that opens the deck list.

use components::{Button, ButtonVariant};
use dioxus::prelude::*;
use zwipe_ui::deck;
use zwipe_ui::screens::home::{self, Decks};

/// The panel between the hero and the Color map: the title bar, then the
/// deck count in one of the hero's stat tiles beside an Open decks button.
/// A failed fetch with nothing cached shows a muted line over the button
/// instead; while the fetch is out the whole panel is a ghost. Home's model
/// decides which.
#[component]
pub fn HomeDecks(decks: Decks, onopen: EventHandler) -> Element {
    rsx! {
        div { class: "container-sm text-center flex-col home-flavor home-decks content-enter-delayed",
            match decks {
                Decks::Count(count) => rsx! {
                    PanelHeader {}
                    div { class: "home-decks-row",
                        section { class: "stats-strip home-decks-strip",
                            div { class: "stat",
                                span { class: "stat-num", "{count}" }
                                span { class: "stat-label", {deck::deck_unit(count)} }
                            }
                        }
                        OpenDecksButton { onclick: onopen }
                    }
                },
                Decks::Failed => rsx! {
                    PanelHeader {}
                    div { class: "message-empty", style: "padding: 1rem;", {home::DECKS_FAILED} }
                    div { class: "home-panel-actions", OpenDecksButton { onclick: onopen } }
                },
                Decks::Loading => rsx! {
                    // The title as a bar, then the tile's shape with bars for
                    // its number and label, and a chip the button's size.
                    div { class: "skeleton-card-header home-panel-header", style: "display: flex;",
                        div { class: "skeleton-bar skeleton-section-label", style: "width: 22%; margin-bottom: 0;" }
                    }
                    div { class: "home-decks-row",
                        section { class: "stats-strip home-decks-strip",
                            div { class: "stat",
                                div { class: "skeleton-bar skeleton-stat-num" }
                                div { class: "skeleton-bar skeleton-stat-label" }
                            }
                        }
                        div { class: "skeleton-bar skeleton-chip skeleton-chip-xl" }
                    }
                },
            }
        }
    }
}

/// The panel's title bar, centered like the Color map's.
#[component]
fn PanelHeader() -> Element {
    rsx! {
        div { class: "settings-list-header home-panel-header",
            span { class: "settings-list-title", {home::DECKS} }
        }
    }
}

/// Opens the deck list.
#[component]
fn OpenDecksButton(onclick: EventHandler) -> Element {
    rsx! {
        Button {
            variant: ButtonVariant::Action,
            onclick: move |_| onclick.call(()),
            {home::OPEN_DECKS}
        }
    }
}
