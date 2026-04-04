//! Deck entity and related operations — server-side error types only.
//!
//! Domain types (Deck, DeckEntry, DeckProfile, etc.) live in zwipe-core.

/// Create deck profile operation.
pub mod create_deck_profile;
/// Delete deck operation.
pub mod delete_deck;
/// Get complete deck operation (profile + cards).
pub mod get_deck;
/// Get tokens produced by all cards in a deck.
pub mod get_deck_tokens;
/// Get single deck profile operation.
pub mod get_deck_profile;
/// Update deck profile operation.
pub mod update_deck_profile;
