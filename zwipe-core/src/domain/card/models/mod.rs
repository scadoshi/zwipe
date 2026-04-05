//! Card domain types shared between frontend and backend.

/// User-specific card metadata (favorites, notes - future expansion).
pub mod card_profile;
/// Mechanical category classification (ramp, draw, removal, etc.).
pub mod mechanical_category;
/// Scryfall API data models.
pub mod scryfall_data;
/// Card search with comprehensive filtering.
pub mod search_card;

use card_profile::CardProfile;
use scryfall_data::ScryfallData;
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

impl Card {
    /// Creates a new card from profile and Scryfall data.
    pub fn new(card_profile: CardProfile, scryfall_data: ScryfallData) -> Self {
        Self {
            card_profile,
            scryfall_data,
        }
    }
}
