//! A deck card's editing buttons, shared by the expanded [`CardRow`] and
//! the card details drawer the deck page's stacks open: the Star and Flag
//! with the row's other icons, the quantity stepper at the end of their
//! line, and the board moves in the word buttons' block under it.
//!
//! [`CardRow`]: crate::CardRow

use dioxus::prelude::*;
use zwipe_core::domain::deck::Board;

use crate::copy_room::can_add_copy;
use crate::{FlagButton, StarButton};

/// A deck card's editing actions, for a host that lays them out with its
/// own buttons, such as the card details panel's foot: the quantity stepper
/// when `on_qty_change` is given, the Star and Flag for `mvp` and `flagged`,
/// and the moves off `current_board`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CardEdit {
    /// The card's copies.
    pub qty: i32,
    /// Changes the copies by the step given.
    pub on_qty_change: Option<EventHandler<i32>>,
    /// Whether the card is starred, when it can be.
    pub mvp: Option<bool>,
    /// Toggles the star.
    pub on_toggle_mvp: Option<EventHandler<()>>,
    /// Whether the card is flagged, when it can be.
    pub flagged: Option<bool>,
    /// Toggles the flag.
    pub on_toggle_flag: Option<EventHandler<()>>,
    /// The card's board, the main board when `None`.
    pub current_board: Option<Board>,
    /// Moves the card to another board.
    pub on_move_to: EventHandler<Board>,
}

/// The editing buttons as one `card-action-row`: the [`CardMarkButtons`] in a
/// `card-action-group`, then the [`CardStepper`] when `on_qty_change` is
/// given, at the line's far end on a phone.
#[component]
pub fn CardEditButtons(
    qty: i32,
    #[props(default)] on_qty_change: Option<EventHandler<i32>>,
    #[props(default)] mvp: Option<bool>,
    #[props(default)] on_toggle_mvp: Option<EventHandler<()>>,
    #[props(default)] flagged: Option<bool>,
    #[props(default)] on_toggle_flag: Option<EventHandler<()>>,
) -> Element {
    rsx! {
        div { class: "card-action-row card-action-split",
            span { class: "card-action-group",
                CardMarkButtons { mvp, on_toggle_mvp, flagged, on_toggle_flag }
            }
            if let Some(on_qty_change) = on_qty_change {
                CardStepper { qty, on_qty_change }
            }
        }
    }
}

/// The quantity stepper: `-`, the count and `+` in one box, so a full row
/// wraps it whole, its `+` disabled once the card can take no more copies
/// ([`can_add_copy`]). Each is a `card-action-btn` that stops its click
/// there, so a row under it stays as it is.
#[component]
pub(crate) fn CardStepper(qty: i32, on_qty_change: EventHandler<i32>) -> Element {
    rsx! {
        span { class: "card-action-stepper",
            button {
                class: "card-action-btn",
                onclick: move |evt| {
                    evt.stop_propagation();
                    on_qty_change.call(-1);
                },
                "-"
            }
            span { class: "card-action-count", "{qty}" }
            button {
                class: "card-action-btn",
                disabled: !can_add_copy(qty),
                onclick: move |evt| {
                    evt.stop_propagation();
                    on_qty_change.call(1);
                },
                "+"
            }
        }
    }
}

/// The [`StarButton`] when `mvp` and `on_toggle_mvp` are given, then the
/// [`FlagButton`] when `flagged` and `on_toggle_flag` are.
#[component]
pub(crate) fn CardMarkButtons(
    #[props(default)] mvp: Option<bool>,
    #[props(default)] on_toggle_mvp: Option<EventHandler<()>>,
    #[props(default)] flagged: Option<bool>,
    #[props(default)] on_toggle_flag: Option<EventHandler<()>>,
) -> Element {
    rsx! {
        if let (Some(is_mvp), Some(handler)) = (mvp, on_toggle_mvp) {
            StarButton { starred: is_mvp, on_toggle: handler }
        }
        if let (Some(is_flagged), Some(handler)) = (flagged, on_toggle_flag) {
            FlagButton { flagged: is_flagged, on_toggle: handler }
        }
    }
}

/// The moves off `current_board` (the main board when `None`) to the other
/// two, as one `card-action-words` row.
#[component]
pub fn CardBoardMoves(current_board: Option<Board>, on_move_to: EventHandler<Board>) -> Element {
    rsx! {
        div { class: "card-action-row card-action-words",
            BoardMoveButtons { current_board, on_move_to }
        }
    }
}

/// The two move buttons of [`CardBoardMoves`], for a host's own row.
#[component]
pub(crate) fn BoardMoveButtons(
    current_board: Option<Board>,
    on_move_to: EventHandler<Board>,
) -> Element {
    let targets = match current_board.unwrap_or(Board::Deck) {
        Board::Deck => [Board::Maybeboard, Board::Sideboard],
        Board::Maybeboard => [Board::Deck, Board::Sideboard],
        Board::Sideboard => [Board::Deck, Board::Maybeboard],
    };
    rsx! {
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

/// The button that moves a card to `board`.
const fn move_label(board: Board) -> &'static str {
    match board {
        Board::Deck => "To mainboard",
        Board::Maybeboard => "To maybeboard",
        Board::Sideboard => "To sideboard",
    }
}
