//! The deck cards page's keyboard cursor (`context/plans/deck_keyboard_nav.md`): which card a
//! move lands on, given the cards as the page draws them, in columns. orangui-core's
//! [`grid_cursor`](orangui_core::grid_cursor) decides; here a card is named by its Scryfall id.

pub use orangui_core::grid_cursor::{Cell, MOVE_KEYS, Move, move_for};
use uuid::Uuid;

/// One card the cursor can stop on, as drawn, named by its Scryfall id.
pub type Stop<I = Uuid> = orangui_core::grid_cursor::Stop<I>;

/// The cards as the page draws them: columns left to right, each card top to bottom.
pub type Snapshot<I = Uuid> = orangui_core::grid_cursor::Snapshot<I>;
