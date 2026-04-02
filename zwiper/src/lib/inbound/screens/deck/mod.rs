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

/// Deck profile info and warnings section for the view screen.
mod deck_profile_section;
/// Deck stats summary section for the view screen.
mod deck_stats_section;
/// Deck chart visualizations for the view screen.
mod deck_charts;
/// Shared format selector and commander search for create/edit screens.
mod deck_form_fields;
