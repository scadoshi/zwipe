//! What the screens draw, free of terminal and network code.

/// A card as text: faces, mana symbols, rules and stats.
pub(crate) mod card;
/// The swipe stack and the query that fills it.
pub(crate) mod swipe;
/// Orangui's palettes, mapped to the roles the screens draw with.
pub(crate) mod theme;
