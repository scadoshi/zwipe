//! Per-stack swipe action models for undo.
//!
//! Split by stack discipline, not by screen. A linear stack (Add screen's
//! search source) keeps swiped cards behind the cursor, so its actions carry
//! no data — undo steps back onto the very card it undoes. Cycling stacks
//! (maybeboard source, Remove screen) drop the card from the list when a
//! swipe commits, so those variants keep the only surviving copy.

use crate::inbound::components::interactions::swipe::direction::Direction;
use zwipe_core::domain::{card::Card, deck::Board};

/// Behavior every stack action shares: which way the card exited, so undo
/// can animate it re-entering from the same side.
pub trait StackAction {
    /// The direction the card exited when the action committed.
    fn exited(&self) -> Direction;
}

/// Add screen, search stack (linear). Field-less: the swiped card is still
/// in the stack at the position undo rewinds to.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AddAction {
    /// Left swipe — durable skip.
    Skip,
    /// Right swipe — added to the deck.
    Add,
    /// Up swipe — sent to the maybeboard.
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

/// Add screen, maybeboard stack (cycling). Promote removes the card from the
/// stack, so that variant carries it for undo's re-insert.
#[derive(Clone, Debug, PartialEq)]
pub enum MaybeboardAction {
    /// Left swipe — cycle past without acting.
    Skip,
    /// Right swipe — promoted to the deck.
    Promote {
        /// The promoted card (removed from the stack on commit).
        card: Box<Card>,
    },
}

impl StackAction for MaybeboardAction {
    fn exited(&self) -> Direction {
        match self {
            Self::Skip => Direction::Left,
            Self::Promote { .. } => Direction::Right,
        }
    }
}

/// Remove screen (cycling). Remove and board moves drop the card from the
/// displayed stack, so those variants carry it for undo's re-insert.
#[derive(Clone, Debug, PartialEq)]
pub enum RemoveAction {
    /// Left swipe — keep the card, cycle past.
    Keep,
    /// Right swipe — removed from the deck.
    Remove {
        /// The removed card (dropped from stack and deck on commit).
        card: Box<Card>,
    },
    /// Up swipe — moved between boards. `from` lets undo restore the
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

/// Oracle-tag examples browse (linear, read-only). Field-less like `AddAction`:
/// the browsed card stays in the stack at the position "back" rewinds to. The
/// only committing gesture is a left swipe (advance), so undo always re-enters
/// from the left.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BrowseAction {
    /// Left swipe — advanced to the next example card.
    Next,
}

impl StackAction for BrowseAction {
    fn exited(&self) -> Direction {
        match self {
            Self::Next => Direction::Left,
        }
    }
}

/// Maximum cards to keep in memory before requiring refresh.
pub const MAX_CARDS_IN_STACK: usize = 500;

/// Warning threshold - show toast when approaching limit.
pub const CARDS_WARNING_THRESHOLD: usize = 400;
