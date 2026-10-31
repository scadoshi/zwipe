//! What the screens draw, free of terminal and network code.

/// The account forms' validation and copy.
pub(crate) mod account;
/// A card as text: faces, mana symbols, rules and stats.
pub(crate) mod card;
/// A deck's row in the deck list.
pub(crate) mod deck;
/// The logo entrance, ported from orangui.
pub(crate) mod decode;
/// The home screen's counters and color map.
pub(crate) mod home;
/// The swipe stacks the Add and Remove screens share, and their undo actions.
#[expect(dead_code, reason = "the card screens use it next")]
pub(crate) mod stack;
/// The stores the card screens share for the whole run.
#[expect(dead_code, reason = "the card screens use it next")]
pub(crate) mod stores;
/// The swipe stack and the query that fills it.
pub(crate) mod swipe;
/// Orangui's palettes, mapped to the roles the screens draw with.
pub(crate) mod theme;
