//! A list of whole cards under a heading: the card page's several cards that share a name, and
//! the card search's results page. Each card is a one-card stack in the deck page's Stacks grid,
//! and a tap opens it.

use dioxus::prelude::*;
use uuid::Uuid;
use zwipe_core::domain::card::Card;

use crate::{CardSkeleton, CardStack, StackCard};

/// `cards` under the heading `title`, each calling `on_select` with its printing id.
#[component]
pub fn CardResultsGrid(title: String, cards: Vec<Card>, on_select: EventHandler<Uuid>) -> Element {
    rsx! {
        h1 { class: "deck-head-title card-list-title", "{title}" }
        div { class: "deck-stacks card-list-results",
            for listed in cards {
                div { key: "{listed.scryfall_data.id}",
                    CardStack {
                        cards: vec![
                            StackCard {
                                card: listed,
                                qty: 1,
                                face: 0,
                                board: None,
                                starred: false,
                            },
                        ],
                        on_select: move |id: Uuid| on_select.call(id),
                    }
                }
            }
        }
    }
}

/// How many card ghosts [`CardResultsGridGhost`] draws: one row on a wide window, two on a phone.
const GHOST_CARDS: usize = 4;

/// [`CardResultsGrid`] while its cards load: a ghost of the heading over card ghosts in the same
/// grid, with no text, so nothing moves when the cards land.
#[component]
pub fn CardResultsGridGhost() -> Element {
    rsx! {
        div { class: "skeleton-bar card-list-title-ghost", "aria-hidden": "true" }
        div { class: "deck-stacks card-list-results", "aria-busy": "true",
            for i in 0..GHOST_CARDS {
                div { key: "{i}", CardSkeleton {} }
            }
        }
    }
}
