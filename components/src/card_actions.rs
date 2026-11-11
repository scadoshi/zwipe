//! A deck card's editing buttons, shared by the expanded [`CardRow`] and
//! the card details drawer the deck page's stacks open: the quantity
//! stepper and the MVP star in the action bar's row, and the board moves as
//! a row of their own.
//!
//! [`CardRow`]: crate::CardRow

use dioxus::prelude::*;
use zwipe_core::domain::deck::Board;

/// The quantity stepper (`-`, the count, `+`) when `on_qty_change` is
/// given, then Star or Unstar when `mvp` and `on_toggle_mvp` are. Each is a
/// `card-action-btn` that stops its click there, so a row under it stays
/// as it is.
#[component]
pub fn CardEditButtons(
    qty: i32,
    #[props(default)] on_qty_change: Option<EventHandler<i32>>,
    #[props(default)] mvp: Option<bool>,
    #[props(default)] on_toggle_mvp: Option<EventHandler<()>>,
) -> Element {
    rsx! {
        if let Some(handler) = on_qty_change {
            button {
                class: "card-action-btn",
                onclick: move |evt| {
                    evt.stop_propagation();
                    handler.call(-1);
                },
                "-"
            }
            span { class: "card-action-count", "{qty}" }
            button {
                class: "card-action-btn",
                onclick: move |evt| {
                    evt.stop_propagation();
                    handler.call(1);
                },
                "+"
            }
        }
        if let (Some(is_mvp), Some(handler)) = (mvp, on_toggle_mvp) {
            button {
                class: "card-action-btn",
                onclick: move |evt| {
                    evt.stop_propagation();
                    handler.call(());
                },
                if is_mvp { "Unstar" } else { "Star" }
            }
        }
    }
}

/// The moves off `current_board` (the main board when `None`) to the other
/// two, as one `card-action-row`.
#[component]
pub fn CardBoardMoves(current_board: Option<Board>, on_move_to: EventHandler<Board>) -> Element {
    let handler = on_move_to;
    rsx! {
        div { class: "card-action-row",
            match current_board.unwrap_or(Board::Deck) {
                Board::Deck => rsx! {
                    button {
                        class: "card-action-btn",
                        style: "white-space:nowrap;",
                        onclick: move |evt| { evt.stop_propagation(); handler.call(Board::Maybeboard); },
                        "To maybeboard"
                    }
                    button {
                        class: "card-action-btn",
                        style: "white-space:nowrap;",
                        onclick: move |evt| { evt.stop_propagation(); handler.call(Board::Sideboard); },
                        "To sideboard"
                    }
                },
                Board::Maybeboard => rsx! {
                    button {
                        class: "card-action-btn",
                        style: "white-space:nowrap;",
                        onclick: move |evt| { evt.stop_propagation(); handler.call(Board::Deck); },
                        "To mainboard"
                    }
                    button {
                        class: "card-action-btn",
                        style: "white-space:nowrap;",
                        onclick: move |evt| { evt.stop_propagation(); handler.call(Board::Sideboard); },
                        "To sideboard"
                    }
                },
                Board::Sideboard => rsx! {
                    button {
                        class: "card-action-btn",
                        style: "white-space:nowrap;",
                        onclick: move |evt| { evt.stop_propagation(); handler.call(Board::Deck); },
                        "To mainboard"
                    }
                    button {
                        class: "card-action-btn",
                        style: "white-space:nowrap;",
                        onclick: move |evt| { evt.stop_propagation(); handler.call(Board::Maybeboard); },
                        "To maybeboard"
                    }
                },
            }
        }
    }
}
