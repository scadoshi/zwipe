//! Deck management handlers.

/// Deck suppression-clear handler.
pub mod clear_deck_suppressions;
/// Deck clone handler.
pub mod clone_deck;
/// Deck creation handler.
pub mod create_deck_profile;
/// Deck deletion handler.
pub mod delete_deck;
/// Full deck with cards handler.
pub mod get_deck;
/// Archidekt deck import handler.
pub mod import_archidekt;
/// Deck tokens handler.
pub mod get_deck_tokens;
/// Deck metadata handler.
pub mod get_deck_profile;
/// All decks for user handler.
pub mod get_deck_profiles;
/// Deck-aware card search handler (exclusion + synergy ordering).
pub mod search_deck_cards;
/// Single-card skip/unskip suppression handlers.
pub mod skip_deck_card;
/// Deck metadata update handler.
pub mod update_deck_profile;
