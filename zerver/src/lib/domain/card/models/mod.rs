pub mod card_profile;
pub mod create_card;
pub mod get_card;
pub mod get_card_types;
pub mod get_sets;
pub mod helpers;
pub mod scryfall_data;
pub mod search_card;

#[cfg(feature = "zerver")]
pub mod sync_metrics;
use crate::domain::card::models::{card_profile::CardProfile, scryfall_data::ScryfallData};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, PartialEq, Deserialize)]
pub struct Card {
    pub card_profile: CardProfile,
    pub scryfall_data: ScryfallData,
}

#[cfg(feature = "zerver")]
impl Card {
    pub fn new(card_profile: CardProfile, scryfall_data: ScryfallData) -> Self {
        Self {
            card_profile,
            scryfall_data,
        }
    }
}
