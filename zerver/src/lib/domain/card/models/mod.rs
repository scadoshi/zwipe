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
use crate::domain::card::models::{card_profile::CardProfile, scryfall_data::ScryfallData};
use serde::{Deserialize, Serialize};

/// Complete MTG card data combining internal metadata and Scryfall card information.
///
/// Aggregates:
/// - **card_profile**: Internal card metadata (sync timestamps, database ID)
/// - **scryfall_data**: Complete Scryfall card object (~100 fields)
#[derive(Debug, Clone, Serialize, PartialEq, Deserialize)]
pub struct Card {
    /// Internal card metadata and sync information.
    pub card_profile: CardProfile,
    /// Complete Scryfall card data (gameplay, print, and core fields).
    pub scryfall_data: ScryfallData,
}

#[cfg(feature = "zerver")]
impl Card {
    /// Creates a new card from profile and Scryfall data.
    pub fn new(card_profile: CardProfile, scryfall_data: ScryfallData) -> Self {
        Self {
            card_profile,
            scryfall_data,
        }
    }
}
