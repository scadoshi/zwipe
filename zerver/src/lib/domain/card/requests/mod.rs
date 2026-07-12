//! Card request types.
//!
//! This module contains all request/response types for card operations.
//! Entities and value objects live in the sibling [`super::models`] module.

/// Card creation/upsert operations.
pub mod create_card;
/// Get distinct artist names from card database.
pub mod get_artists;
/// Get single/multiple cards operations.
pub mod get_card;
/// Get card profile operation.
pub mod get_card_profile;
/// Get distinct card types from database.
pub mod get_card_types;
/// Get distinct keyword abilities from database.
pub mod get_keywords;
/// Get distinct languages from card database.
pub mod get_languages;
/// Get the oracle tag catalog.
pub mod get_oracle_tags;
/// Get distinct normalized words from oracle text.
pub mod get_oracle_words;
/// Get Scryfall data operations.
pub mod get_scryfall_data;
/// Get distinct set codes/names from card database.
pub mod get_sets;
