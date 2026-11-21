//! A list of whole cards under a heading: the card page's several cards that share a name, and
//! the card search's results page. Each card is a one-card stack in the deck page's Stacks grid,
//! and a tap opens it. The grid carries [`GRID_ATTR`](crate::deck_cursor::GRID_ATTR), so a page with the keyboard cursor
//! ([`use_card_grid_cursor`](crate::deck_cursor::use_card_grid_cursor)) walks it as laid out.

use std::collections::HashMap;

use dioxus::prelude::*;
use uuid::Uuid;
use zwipe_core::domain::card::Card;

use crate::{CardSkeleton, CardStack, StackCard};

/// `cards` under the heading `title`, each calling `on_select` with its printing id and showing
/// the face `faces` holds for it, else its front. The cards from `dealt_from` on, a next page
/// that just landed, deal in a little apart, each by its place in that page.
#[component]
pub fn CardResultsGrid(
    title: String,
    cards: Vec<Card>,
    on_select: EventHandler<Uuid>,
    #[props(default)] faces: HashMap<Uuid, usize>,
    #[props(default)] dealt_from: Option<usize>,
) -> Element {
    rsx! {
        h1 { class: "card-list-title",
            span { class: "chip-note", "{title}" }
        }
        div { class: "deck-stacks card-list-results", "data-cursor-grid": "",
            for (i, listed) in cards.into_iter().enumerate() {
                div {
                    key: "{listed.scryfall_data.id}",
                    class: if dealt(i, dealt_from).is_some() { "card-list-dealt" },
                    style: dealt(i, dealt_from).map(|n| format!("--deal-n: {n}")),
                    CardStack {
                        cards: vec![
                            StackCard {
                                face: faces.get(&listed.scryfall_data.id).copied().unwrap_or(0),
                                card: listed,
                                qty: 1,
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

/// Card `i`'s place in the newest page that starts at `from`, while it is one of that page's.
fn dealt(i: usize, from: Option<usize>) -> Option<usize> {
    i.checked_sub(from?)
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

#[cfg(test)]
mod tests {
    use crate::deck_cursor::GRID_ATTR;

    #[test]
    fn the_grid_is_marked_for_the_cursor() {
        let source = include_str!("card_results_grid.rs");
        assert!(
            source.contains(&format!("\"{GRID_ATTR}\": \"\"")),
            "the results grid carries the cursor's grid mark"
        );
    }
}
