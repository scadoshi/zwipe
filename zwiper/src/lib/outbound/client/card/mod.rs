//! Card data API client operations.
//!
//! Provides traits and implementations for fetching MTG card data:
//! card search, individual card retrieval, and metadata lookups.

/// Fetch all unique artist names.
pub mod get_artists;
/// Fetch a single card by ID.
pub mod get_card;
/// Fetch all card types (creature, instant, etc.).
pub mod get_card_types;
/// Fetch all available languages.
pub mod get_languages;
/// Fetch all card sets.
pub mod get_sets;
/// Search cards with filters.
pub mod search_cards;
