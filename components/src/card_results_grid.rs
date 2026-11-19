//! A list of whole cards under a heading: the card page's several cards that share a name, and
//! the card search's results page. Each card is a one-card stack in the deck page's Stacks grid,
//! and a tap opens it.

use dioxus::prelude::*;
use uuid::Uuid;
use zwipe_core::domain::card::Card;

use crate::{CardStack, StackCard};

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
