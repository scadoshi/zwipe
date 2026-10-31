//! What the screens draw, free of terminal and network code.

/// The Add screen's view of its deck and its search rules.
pub(crate) mod add;
/// Orangui's rolling figures and compact numbers.
pub(crate) mod count_up;
/// The deck cards view's pass, featured strip, quantity bursts and undo rules.
pub(crate) mod deck_cards;
/// Land and budget signals as a deck changes.
pub(crate) mod deck_signals;
/// The logo entrance, ported from orangui.
pub(crate) mod decode;
/// The card filter's sections, fields, pickers and Apply rules.
pub(crate) mod filter;
/// The privacy policy's HTML as text.
pub(crate) mod legal;
pub(crate) mod stack;
/// The stores the card screens share for the whole run.
pub(crate) mod stores;
/// Orangui's palettes, mapped to the roles the screens draw with.
pub(crate) mod theme;
