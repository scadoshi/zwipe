//! Deck entity and related operations.
//!
//! This module provides the complete deck entity (profile + cards) and all
//! deck profile management operations.

/// Create deck profile operation.
pub mod create_deck_profile;
/// Deck warning value object.
pub mod deck_warning;
/// Deck name validation (1-64 chars, no profanity).
pub mod deck_name;
/// Deck profile entity (deck metadata).
pub mod deck_profile;
/// Delete deck operation.
pub mod delete_deck;
/// Deck format classification (Commander, Standard, Modern, etc.).
pub mod format;
/// Get complete deck operation (profile + cards).
pub mod get_deck;
/// Get tokens produced by all cards in a deck.
pub mod get_deck_tokens;
/// Get single deck profile operation.
pub mod get_deck_profile;
/// Deck validation logic (generates warnings).
pub mod validate_deck;
/// Get multiple deck profiles operation (list user's decks).
pub mod get_deck_profiles;
/// Update deck profile operation.
pub mod update_deck_profile;

pub use zwipe_core::domain::deck::{Deck, DeckEntry};
