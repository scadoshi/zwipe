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
/// Get single deck profile operation.
pub mod get_deck_profile;
/// Get tokens produced by all cards in a deck.
pub mod get_deck_tokens;
/// Import a deck from Archidekt.
pub mod import_archidekt;
/// Deck-aware card search (exclusion + synergy ordering).
pub mod search_deck_cards;
/// Deck share operations (share token create/revoke, public shared read).
pub mod share_deck;
/// Skip deck card operation (single durable suppression).
pub mod skip_deck_card;
/// Update deck profile operation.
pub mod update_deck_profile;
