//! A deck card's editing buttons, shared by the expanded [`CardRow`] and
//! the card details drawer the deck page's stacks open: the quantity
//! stepper and the MVP star in the action bar's row, and the board moves as
//! a row of their own.
//!
//! [`CardRow`]: crate::CardRow

use dioxus::prelude::*;
use zwipe_core::domain::deck::Board;

use crate::StarButton;

/// The quantity stepper (`-`, the count, `+`) when `on_qty_change` is
/// given, then the [`StarButton`] when `mvp` and `on_toggle_mvp` are. Each is a
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
            StarButton { starred: is_mvp, on_toggle: handler }
        }
    }
}

/// The moves off `current_board` (the main board when `None`) to the other
/// two, as one `card-action-row`.
#[component]
pub fn CardBoardMoves(current_board: Option<Board>, on_move_to: EventHandler<Board>) -> Element {
    let targets = match current_board.unwrap_or(Board::Deck) {
        Board::Deck => [Board::Maybeboard, Board::Sideboard],
        Board::Maybeboard => [Board::Deck, Board::Sideboard],
        Board::Sideboard => [Board::Deck, Board::Maybeboard],
    };
    rsx! {
        div { class: "card-action-row",
            for target in targets {
                button {
                    key: "{move_label(target)}",
                    class: "card-action-btn",
                    style: "white-space:nowrap;",
                    onclick: move |evt| {
                        evt.stop_propagation();
                        on_move_to.call(target);
                    },
                    {move_label(target)}
                }
            }
        }
    }
}

/// The button that moves a card to `board`.
const fn move_label(board: Board) -> &'static str {
    match board {
        Board::Deck => "To mainboard",
        Board::Maybeboard => "To maybeboard",
        Board::Sideboard => "To sideboard",
    }
}
