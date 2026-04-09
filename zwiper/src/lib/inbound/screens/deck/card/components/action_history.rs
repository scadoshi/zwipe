//! Swipe action history for undo/redo functionality.

use crate::inbound::components::interactions::swipe::direction::Direction;
use zwipe_core::domain::card::Card;

/// The type of swipe action performed on a card.
///
/// Each variant carries the `Direction` the card exited toward so undo can
/// animate the restored card re-entering from the same side.
#[derive(Clone, Debug, PartialEq)]
#[allow(missing_docs)]
pub enum SwipeAction {
    Skip { card: Box<Card>, exited: Direction },
    Do { card: Box<Card>, exited: Direction },
    Maybeboard { card: Box<Card>, exited: Direction },
}

impl SwipeAction {
    /// The direction the card exited when the action committed.
    pub fn exited(&self) -> &Direction {
        match self {
            Self::Skip { exited, .. }
            | Self::Do { exited, .. }
            | Self::Maybeboard { exited, .. } => exited,
        }
    }
}

/// Maximum cards to keep in memory before requiring refresh.
pub const MAX_CARDS_IN_STACK: usize = 1000;

/// Warning threshold - show toast when approaching limit.
pub const CARDS_WARNING_THRESHOLD: usize = 500;
