//! The deck's cards in Stacks while the deck loads, for the app and the web
//! alike: each stack's group chip over its cards, each card after the first
//! tucked under the one above it, as a loaded stack lies. A client shows it
//! in place of its List ghost when the saved View is Stacks.

use dioxus::prelude::*;
use zwipe_ui::deck_cards::{columns, stacks};

use crate::deck_cards_view::LAST_WIDTH;

/// Ghost cards in each stack: the Commander's one, then fuller groups.
const STACK_CARDS: [usize; 4] = [1, 6, 3, 4];

/// Each ghost stack's name, in characters.
const NAME_CHARS: [u8; 4] = [9, 12, 8, 10];

/// The ghost stacks, in [`crate::DeckStacks`]'s layout: with `touch` (phones)
/// two columns, else as many as the width the stacks last measured fits (the
/// loaded stacks' own measure, read here so the two never disagree), at
/// least one stack in each, each stack in the shorter column as they lie.
/// Before any measure, the page's grid, as the loaded stacks start.
#[component]
pub fn DeckStacksGhost(#[props(default)] touch: bool) -> Element {
    let width = LAST_WIDTH();
    let column_total = if touch {
        Some(2)
    } else {
        width.map(columns::column_count)
    };
    // One stack for each column at least, so a wide page's ghost fills its row.
    let count = column_total.unwrap_or(0).max(STACK_CARDS.len());
    let shapes: Vec<(usize, u8)> = STACK_CARDS
        .into_iter()
        .zip(NAME_CHARS)
        .cycle()
        .take(count)
        .collect();
    let stacks: Vec<Element> = shapes
        .iter()
        .enumerate()
        .map(|(i, &(cards, chars))| ghost_stack(i, cards, chars))
        .collect();
    let Some(column_total) = column_total else {
        return rsx! {
            div { class: "deck-stacks", "aria-hidden": "true",
                for stack in stacks {
                    {stack}
                }
            }
        };
    };
    let heights: Vec<u32> = shapes
        .iter()
        .map(|&(cards, _)| columns::stack_height(cards, false))
        .collect();
    let mut laid: Vec<Vec<Element>> = vec![Vec::new(); column_total.max(1)];
    for (column, stack) in columns::place(&heights, column_total)
        .into_iter()
        .zip(stacks)
    {
        if let Some(held) = laid.get_mut(column) {
            held.push(stack);
        }
    }
    let class = if touch {
        "deck-stacks deck-stacks-columns deck-stacks-touch"
    } else {
        "deck-stacks deck-stacks-columns"
    };
    rsx! {
        div { class, "aria-hidden": "true",
            for (at , held) in laid.into_iter().enumerate() {
                div { key: "{at}", class: "deck-stacks-column",
                    for stack in held {
                        {stack}
                    }
                }
            }
        }
    }
}

/// One ghost stack of `cards` under a name `chars` wide.
fn ghost_stack(key: usize, cards: usize, chars: u8) -> Element {
    rsx! {
        div { key: "{key}", class: "deck-stack",
            div { class: "card-group deck-group",
                div { class: "card-group-header group-collapsible",
                    span { class: "skeleton-bar deck-stacks-ghost-name", style: "width: {chars}ch;" }
                }
                div { class: "card-stack",
                    for (c , tucked) in stacks::tucked(cards).into_iter().enumerate() {
                        div {
                            key: "{c}",
                            class: if tucked { "card-stack-slot tucked" } else { "card-stack-slot" },
                            div { class: "card-stack-card skeleton-bar deck-stacks-ghost-card" }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn html(touch: bool) -> String {
        let mut dom = VirtualDom::new_with_props(DeckStacksGhost, DeckStacksGhostProps { touch });
        dom.rebuild_in_place();
        dioxus_ssr::render(&dom)
    }

    #[test]
    fn a_ghost_stack_tucks_every_card_after_its_first() {
        let page = html(false);
        let cards: usize = STACK_CARDS.iter().sum();
        assert_eq!(
            page.matches("deck-stacks-ghost-card").count(),
            cards,
            "{page}"
        );
        let tucked: usize = STACK_CARDS.iter().map(|n| n.saturating_sub(1)).sum();
        assert_eq!(
            page.matches("card-stack-slot tucked").count(),
            tucked,
            "{page}"
        );
        assert!(!page.contains("deck-stacks-touch"), "{page}");
    }

    #[test]
    fn on_a_phone_the_ghost_lies_in_two_columns() {
        let page = html(true);
        assert!(
            page.contains("deck-stacks deck-stacks-columns deck-stacks-touch"),
            "{page}"
        );
        assert_eq!(
            page.matches(r#"class="deck-stacks-column""#).count(),
            2,
            "{page}"
        );
    }
}
