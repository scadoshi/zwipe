//! The deck's cards in Stacks while the deck loads, for the app and the web
//! alike: each stack's group chip over its cards, each card after the first
//! tucked under the one above it, as a loaded stack lies. A client shows it
//! in place of its List ghost when the saved View is Stacks.

use dioxus::prelude::*;
use zwipe_ui::deck_cards::{columns, stacks};

/// Ghost cards in each stack: the Commander's one, then fuller groups.
const STACK_CARDS: [usize; 4] = [1, 6, 3, 4];

/// Each ghost stack's name, in characters.
const NAME_CHARS: [u8; 4] = [9, 12, 8, 10];

/// The ghost stacks, in [`crate::DeckStacks`]'s layout: with `touch` (phones)
/// two columns, each stack in the shorter one as the loaded stacks lie; else
/// the page's grid.
#[component]
pub fn DeckStacksGhost(#[props(default)] touch: bool) -> Element {
    let stacks: Vec<Element> = STACK_CARDS
        .into_iter()
        .zip(NAME_CHARS)
        .enumerate()
        .map(|(i, (cards, chars))| ghost_stack(i, cards, chars))
        .collect();
    if !touch {
        return rsx! {
            div { class: "deck-stacks", "aria-hidden": "true",
                for stack in stacks {
                    {stack}
                }
            }
        };
    }
    let heights: Vec<u32> = STACK_CARDS
        .iter()
        .map(|&cards| columns::stack_height(cards, false))
        .collect();
    let mut laid: Vec<Vec<Element>> = vec![Vec::new(); 2];
    for (column, stack) in columns::place(&heights, 2).into_iter().zip(stacks) {
        if let Some(held) = laid.get_mut(column) {
            held.push(stack);
        }
    }
    rsx! {
        div { class: "deck-stacks deck-stacks-columns deck-stacks-touch", "aria-hidden": "true",
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
