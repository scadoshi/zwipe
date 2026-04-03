/// User-specific card metadata (favorites, notes - future expansion).
pub mod card_profile;
/// Card creation/upsert operations.
pub mod create_card;
/// Get distinct artist names from card database.
pub mod get_artists;
/// Get single/multiple cards operations.
pub mod get_card;
/// Get distinct card types from database.
pub mod get_card_types;
/// Get distinct keyword abilities from database.
pub mod get_keywords;
/// Get distinct normalized words from oracle text.
pub mod get_oracle_words;
/// Get distinct languages from card database.
pub mod get_languages;
/// Get distinct set codes/names from card database.
pub mod get_sets;
/// Helper traits and utilities for card operations.
pub mod helpers;
/// Scryfall API data models and operations.
pub mod scryfall_data;
/// Card search with comprehensive filtering.
pub mod search_card;

/// Sync metrics tracking for Scryfall bulk data operations.
#[cfg(feature = "zerver")]
pub mod sync_metrics;

pub use zwipe_core::domain::card::Card;
