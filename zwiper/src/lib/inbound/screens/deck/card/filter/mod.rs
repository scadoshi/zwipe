//! Card search filter components.
//!
//! Provides UI components for building card search filters by various attributes.

/// Artist filter.
pub mod artist;
/// Deck-aware filter data extraction.
pub mod deck_cards;
/// Combat stats filter (power/toughness).
pub mod combat;
/// Filter configuration accordion.
pub mod config;
/// Filter mode toggle (exact vs range).
pub mod filter_mode;
/// Match mode toggle (any vs all).
pub mod match_mode;
/// Flavor text filter.
pub mod flavor_text;
/// Mana cost/color filter.
pub mod mana;
/// Card name filter.
pub mod name;
/// Oracle text, oracle words, and keywords filter.
pub mod oracle_text;
/// Card rarity filter.
pub mod rarity;
/// Card set filter.
pub mod set;
/// Sort order selection.
pub mod sort;
/// Card type filter.
pub mod types;
