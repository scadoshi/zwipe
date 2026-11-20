//! Deck management screens.
//!
//! Provides screens for creating, viewing, editing, and listing decks.

/// Card management within decks (add/remove cards).
pub mod card;
/// The Commanders screen: the commander maybeboard, then the suggested commanders.
pub mod commander_maybeboard;
/// The Commanders screen's Suggested list, under the Maybeboard.
pub(crate) mod commanders_suggested;
/// Create new deck screen.
pub mod create;
/// Edit existing deck screen.
pub mod edit;
/// Export deck as plain-text decklist screen.
pub mod export;
/// Import cards from plain-text decklist screen.
pub mod import;
/// Deck list screen showing all user's decks.
pub mod list;
/// Leaving a deck screen whose deck is gone.
pub(crate) mod missing_deck;
/// Playstyle screen: the user's decks summed up over suggested
/// commanders ranked against them.
pub mod suggested_commanders;
/// View deck details screen.
pub mod view;

/// Extracted components for deck screens.
pub(crate) mod components;
