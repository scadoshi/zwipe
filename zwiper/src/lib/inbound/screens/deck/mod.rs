//! Deck management screens.
//!
//! Provides screens for creating, viewing, editing, and listing decks.

/// Card management within decks (add/remove cards).
pub mod card;
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
/// View deck details screen.
pub mod view;

/// Extracted components for deck screens.
mod components;
