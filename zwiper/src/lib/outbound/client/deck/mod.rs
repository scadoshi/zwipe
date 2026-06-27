//! Deck API client operations.
//!
//! Provides traits and implementations for deck CRUD operations:
//! create, read, update, and delete decks.

/// Clone an existing deck with a new name.
pub mod clone_deck;
/// Create a new deck.
pub mod create_deck;
/// Delete an existing deck.
pub mod delete_deck;
/// Fetch a deck with all its cards.
pub mod get_deck;
/// Fetch a single deck profile (metadata only).
pub mod get_deck_profile;
/// Fetch all deck profiles for the current user.
pub mod get_deck_profiles;
/// Fetch tokens produced by a deck's cards.
pub mod get_deck_tokens;
/// Import a deck from an Archidekt URL.
pub mod import_archidekt_deck;
/// Deck-aware card search (server-side exclusion + synergy default order).
pub mod search_deck_cards;
/// Update deck profile metadata.
pub mod update_deck_profile;
