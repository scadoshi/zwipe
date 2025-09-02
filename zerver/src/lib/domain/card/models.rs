use crate::domain::card::models::{card_profile::CardProfile, scryfall_data::ScryfallData};

pub mod card_profile;
pub mod scryfall_data;
pub mod sync_metrics;

#[derive(Debug, Clone)]
pub struct Card {
    card_profile: CardProfile,
    scryfall_data: ScryfallData,
}

impl Card {
    pub fn new(card_profile: CardProfile, scryfall_data: ScryfallData) -> Self {
        Self {
            card_profile,
            scryfall_data,
        }
    }
}
