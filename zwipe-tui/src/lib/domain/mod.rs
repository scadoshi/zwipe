//! What the screens draw, free of terminal and network code.

/// The account forms' validation and copy.
pub(crate) mod account;
/// A card as text: faces, mana symbols, rules and stats.
pub(crate) mod card;
/// The changelog cache and how its releases are filed.
pub(crate) mod changelog;
/// Rolling counters, ported from orangui.
pub(crate) mod count_up;
/// A deck's row in the deck list.
pub(crate) mod deck;
/// The logo entrance, ported from orangui.
pub(crate) mod decode;
/// The home screen's counters and color map.
pub(crate) mod home;
/// The privacy policy's HTML as text.
pub(crate) mod legal;
/// The swipe stack and the query that fills it.
pub(crate) mod swipe;
/// Orangui's palettes, mapped to the roles the screens draw with.
pub(crate) mod theme;
