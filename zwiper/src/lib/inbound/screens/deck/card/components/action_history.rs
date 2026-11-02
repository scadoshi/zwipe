//! Per-stack swipe action models for undo.
//!
//! Split by stack discipline, not by screen. A linear stack (Add screen's
//! search source) keeps swiped cards behind the cursor, so its actions carry
//! no data: undo steps back onto the very card it undoes. A cycling stack
//! (the maybeboard source) drops the card from the list when a swipe
//! commits, so that variant keeps the only surviving copy. The Remove
//! screen's stack lives in `zwipe_ui::screens::remove_cards`.

use zwipe_core::domain::card::Card;

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

/// Add screen, maybeboard stack (cycling). Promote removes the card from the
/// stack, so that variant carries it for undo's re-insert.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum MaybeboardAction {
    /// Left swipe: cycle past without acting.
    Skip,
    /// Right swipe: promoted to the deck.
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

/// Oracle-tag examples browse (linear, read-only). Field-less like `AddAction`:
/// the browsed card stays in the stack at the position "back" rewinds to. The
/// only committing gesture is a left swipe (advance), so undo always re-enters
/// from the left.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum BrowseAction {
    /// Left swipe: advanced to the next example card.
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
pub(crate) const MAX_CARDS_IN_STACK: usize = 500;

/// Warning threshold: show a toast when approaching the limit.
pub(crate) const CARDS_WARNING_THRESHOLD: usize = 400;
