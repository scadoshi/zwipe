//! Deck card composition handlers.

/// Add card to deck handler.
pub mod create_deck_card;
/// Remove card from deck handler.
pub mod delete_deck_card;
/// Error conversions for deck card lookups.
pub mod get_deck_card;
/// Import cards from plain-text decklist handler.
pub mod import_deck_cards;
/// Idempotent (PATCH, absolute-quantity) card update handler.
pub mod patch_deck_card;
