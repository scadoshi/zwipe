//! One group of a deck's cards as a stack: the cards overlap, each tucked
//! under the one above it so only its title strip shows, the last in full.
//! Which card tucks, which move down to reveal one, and which show in full
//! come from zwipe-ui's [`stacks::tucked`], [`stacks::shifted`] and
//! [`stacks::exposed`], so the phones' stacks do the same.

use dioxus::prelude::*;
use uuid::Uuid;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::scryfall_data::ImageSize;
use zwipe_ui::deck_cards::stacks;

/// One card in a [`CardStack`], with its copies in the deck.
#[derive(Debug, Clone, PartialEq)]
pub struct StackCard {
    /// The card.
    pub card: Card,
    /// Its copies; more than one shows a badge.
    pub qty: i32,
}

/// The stack of `cards`, top to bottom. `revealed` shows in full, the cards
/// after it moving down to make room; `open` outlines the card whose details
/// are up while it shows in full.
/// A pointer over a card or focus on it calls `on_hover` and leaving calls
/// `on_leave`; a click or Enter calls `on_select`.
#[component]
pub fn CardStack(
    cards: Vec<StackCard>,
    #[props(default)] revealed: Option<Uuid>,
    #[props(default)] open: Option<Uuid>,
    on_hover: EventHandler<Uuid>,
    on_leave: EventHandler<Uuid>,
    on_select: EventHandler<Uuid>,
) -> Element {
    let at = revealed.and_then(|r| cards.iter().position(|c| c.card.scryfall_data.id == r));
    let tucks = stacks::tucked(cards.len());
    let shifts = stacks::shifted(cards.len(), at);
    let full = stacks::exposed(cards.len(), at);
    rsx! {
        div { class: "card-stack", role: "list",
            for (((StackCard { card, qty }, tucked), shifted), exposed) in cards.into_iter().zip(tucks).zip(shifts).zip(full) {
                {
                    let id = card.scryfall_data.id;
                    let name = card.scryfall_data.name.clone();
                    let image = card.scryfall_data.primary_image_url(ImageSize::Normal).map(str::to_owned);
                    let mut class = String::from("card-stack-slot");
                    if tucked {
                        class.push_str(" tucked");
                    }
                    if shifted {
                        class.push_str(" shifted");
                    }
                    if exposed && open == Some(id) {
                        class.push_str(" open");
                    }
                    let label = if qty > 1 { format!("{name}, {qty} copies") } else { name.clone() };
                    rsx! {
                        div { key: "{id}", role: "listitem", class: "{class}",
                            button {
                                r#type: "button",
                                class: "card-stack-card",
                                "aria-label": "{label}",
                                onmouseenter: move |_| on_hover.call(id),
                                onmouseleave: move |_| on_leave.call(id),
                                onfocus: move |_| on_hover.call(id),
                                onblur: move |_| on_leave.call(id),
                                onclick: move |_| on_select.call(id),
                                if let Some(src) = image {
                                    img { src: "{src}", alt: "", loading: "lazy", draggable: "false" }
                                } else {
                                    span { class: "card-stack-name", "{name}" }
                                }
                                if qty > 1 {
                                    span { class: "card-stack-qty", "\u{d7}{qty}" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
