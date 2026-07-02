//! Deck entity and related operations — server-side error types only.
//!
//! Domain types (Deck, DeckEntry, DeckProfile, etc.) live in zwipe-core.

/// Clear deck suppressions operation.
pub mod clear_deck_suppressions;
/// Clone deck operation.
pub mod clone_deck;
/// Create deck profile operation.
pub mod create_deck_profile;
/// Delete deck operation.
pub mod delete_deck;
/// Get complete deck operation (profile + cards).
pub mod get_deck;
/// Import a deck from Archidekt.
pub mod import_archidekt;
/// Get tokens produced by all cards in a deck.
pub mod get_deck_tokens;
/// Get single deck profile operation.
pub mod get_deck_profile;
/// Deck-aware card search (exclusion + synergy ordering).
pub mod search_deck_cards;
/// Update deck profile operation.
pub mod update_deck_profile;
