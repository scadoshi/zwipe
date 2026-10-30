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
/// The swipe stack and the query that fills it.
pub(crate) mod swipe;
/// Orangui's palettes, mapped to the roles the screens draw with.
pub(crate) mod theme;
