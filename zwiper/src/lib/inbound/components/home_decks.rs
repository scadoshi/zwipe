//! Home's Decks row: the user's deck count as a chip beside an Open button
//! that opens the deck list.

use components::{Button, ButtonVariant};
use dioxus::prelude::*;
use zwipe_ui::screens::home::{self, Decks};

/// The row between the hero and the Color map, in the settings-row style:
/// the Decks label on the left, the count chip and the Open button on the
/// right. A failed fetch with nothing cached puts a `?` in the chip with a
/// muted hint beside it; while the fetch is out the chip and button ghost.
/// Home's model decides which.
#[component]
pub fn HomeDecks(decks: Decks, onopen: EventHandler) -> Element {
    rsx! {
        div { class: "container-sm home-decks content-enter-delayed",
            div { class: "settings-list",
                div { class: "settings-row",
                    span { class: "settings-row-label", {home::DECKS} }
                    div { class: "settings-row-value",
                        match decks {
                            Decks::Count(count) => rsx! {
                                span { class: "stat-chip", "{count}" }
                                OpenDecksButton { onclick: onopen }
                            },
                            Decks::Failed => rsx! {
                                span { class: "home-decks-hint", {home::DECKS_FAILED} }
                                span { class: "stat-chip stat-chip-muted", "?" }
                                OpenDecksButton { onclick: onopen }
                            },
                            Decks::Loading => rsx! {
                                div { class: "skeleton-bar skeleton-chip home-decks-skeleton-count" }
                                div { class: "skeleton-bar skeleton-chip home-decks-skeleton-open" }
                            },
                        }
                    }
                }
            }
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
