//! Swipe action history for undo/redo functionality.

use zwipe_core::domain::card::Card;

/// The type of swipe action performed on a card.
#[derive(Clone, Debug, PartialEq)]
#[allow(missing_docs)]
pub enum SwipeAction {
    Skip(Box<Card>),
    Do(Box<Card>),
}

/// Maximum cards to keep in memory before requiring refresh.
pub const MAX_CARDS_IN_STACK: usize = 1000;

/// Warning threshold - show toast when approaching limit.
pub const CARDS_WARNING_THRESHOLD: usize = 500;
