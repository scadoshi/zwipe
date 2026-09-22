//! Deck API client operations.
//!
//! Provides traits and implementations for deck CRUD operations:
//! create, read, update, and delete decks.

/// Clear a deck's suppression set (skipped/removed cards).
pub mod clear_deck_suppressions;
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
/// Fetch the deck-tag catalog (slug, label, description, seed otags).
pub mod get_deck_tags;
/// Fetch tokens produced by a deck's cards.
pub mod get_deck_tokens;
/// Import a deck from an Archidekt URL.
pub mod import_archidekt_deck;
/// Deck-aware card search (server-side exclusion + synergy default order).
pub mod search_deck_cards;
/// Share / unshare a deck (public link token management).
pub mod share_deck;
/// Post a single durable skip (and its undo) for a deck.
pub mod skip_deck_card;
/// Update deck profile metadata.
pub mod update_deck_profile;
