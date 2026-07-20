//! Card search filter components.
//!
//! Provides UI components for building card search filters by various attributes.

/// Artist filter.
pub mod artist;
/// Shared bottom-sheet filter accordion.
pub(crate) mod card_filter_sheet;
/// Card role filter.
pub mod card_role;
/// Combat stats filter (power/toughness).
pub mod combat;
/// Filter configuration accordion.
pub mod config;
/// Deck-aware filter data extraction.
pub mod deck_cards;
/// Filter mode toggle (exact vs range).
pub mod filter_mode;
/// Flavor text filter.
pub mod flavor_text;
/// Format legality filter.
pub mod format;
/// Mana cost/color filter.
pub mod mana;
/// Match mode toggle (any vs all).
pub mod match_mode;
/// Card name filter.
pub mod name;
/// Oracle tags filter (Scryfall community functional tags).
pub mod oracle_tags;
/// Oracle text, oracle words, and keywords filter.
pub mod oracle_text;
/// Price range filter (min/max in a selected currency).
pub mod price;
/// Card rarity filter.
pub mod rarity;
/// Card set filter.
pub mod set;
/// Sort order selection.
pub mod sort;
/// Card type filter.
pub mod types;
