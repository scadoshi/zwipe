//! Per-stack swipe action models for undo.
//!
//! Split by stack discipline, not by screen. `AddAction` is the Add
//! screen's search swipe as its parked stacks keep it; the screen's own
//! stacks live in `zwipe_ui::screens::add_cards`. A cycling stack (the Remove
//! screen) drops the card from the list when a swipe commits, so those
//! variants keep the only surviving copy.

use zwipe_core::domain::card::Card;
use zwipe_core::domain::deck::Board;

use crate::inbound::components::interactions::swipe::direction::Direction;

/// Behavior every stack action shares: which way the card exited, so undo
/// can animate it re-entering from the same side.
pub(crate) trait StackAction {
    /// The direction the card exited when the action committed.
    fn exited(&self) -> Direction;
}

/// Add screen, search stack (linear). Field-less: the swiped card is still
/// in the stack at the position undo rewinds to.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum AddAction {
    /// Left swipe: durable skip.
    Skip,
    /// Right swipe: added to the deck.
    Add,
    /// Up swipe: sent to the maybeboard.
    Maybe,
}

impl StackAction for AddAction {
    fn exited(&self) -> Direction {
        match self {
            Self::Skip => Direction::Left,
            Self::Add => Direction::Right,
            Self::Maybe => Direction::Up,
        }
    }
}

/// Remove screen (cycling). Remove and board moves drop the card from the
/// displayed stack, so those variants carry it for undo's re-insert.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum RemoveAction {
    /// Left swipe: keep the card, cycle past.
    Keep,
    /// Right swipe: removed from the deck.
    Remove {
        /// The removed card (dropped from stack and deck on commit).
        card: Box<Card>,
        /// Board the card sat on, so undo puts it back there.
        board: Board,
        /// Copies the deck held, so undo restores all of them.
        quantity: i32,
    },
    /// Up swipe: moved between boards. `from` lets undo restore the
    /// original board.
    MoveBoard {
        /// The moved card (dropped from the displayed stack on commit).
        card: Box<Card>,
        /// Board the card came from.
        from: Board,
        /// Board the card moved to.
        to: Board,
    },
}

impl StackAction for RemoveAction {
    fn exited(&self) -> Direction {
        match self {
            Self::Keep => Direction::Left,
            Self::Remove { .. } => Direction::Right,
            Self::MoveBoard { .. } => Direction::Up,
        }
    }
}
