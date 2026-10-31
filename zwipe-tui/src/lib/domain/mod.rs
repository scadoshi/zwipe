//! What the screens draw, free of terminal and network code.

/// The account forms' validation and copy.
pub(crate) mod account;
/// The Add screen's view of its deck and its search rules.
pub(crate) mod add;
/// A card as text: faces, mana symbols, rules and stats.
pub(crate) mod card;
/// A deck's row in the deck list.
pub(crate) mod deck;
/// The deck cards view's pass, featured strip, quantity bursts and undo rules.
pub(crate) mod deck_cards;
/// Land and budget signals as a deck changes.
pub(crate) mod deck_signals;
/// The logo entrance, ported from orangui.
pub(crate) mod decode;
/// The card filter's sections, fields, pickers and Apply rules.
pub(crate) mod filter;
/// The home screen's counters and color map.
pub(crate) mod home;
/// The swipe stacks the Add and Remove screens share, and their undo actions.
pub(crate) mod stack;
/// The stores the card screens share for the whole run.
pub(crate) mod stores;
/// Orangui's palettes, mapped to the roles the screens draw with.
pub(crate) mod theme;
