#[cfg(feature = "zerver")]
use crate::domain::card::models::{card_profile::CardProfile, scryfall_data::ScryfallData, Card};
#[cfg(feature = "zerver")]
use std::collections::HashMap;
#[cfg(feature = "zerver")]
use uuid::Uuid;

#[cfg(feature = "zerver")]
pub trait SleeveScryfallData {
    fn sleeve(self, card_profiles: Vec<CardProfile>) -> Vec<Card>;
}

#[cfg(feature = "zerver")]
impl SleeveScryfallData for Vec<ScryfallData> {
    fn sleeve(self, card_profiles: Vec<CardProfile>) -> Vec<Card> {
        let mut data_map: HashMap<Uuid, ScryfallData> = self
            .into_iter()
            .map(|sfd| (sfd.id.to_owned(), sfd))
            .collect();

        card_profiles
            .into_iter()
            .filter_map(|cp| {
                data_map
                    .remove(&cp.scryfall_data_id)
                    .map(|sfd| Card::new(cp, sfd))
            })
            .collect::<Vec<Card>>()
    }
}

#[cfg(feature = "zerver")]
pub trait SleeveCardProfile {
    fn sleeve(self, scryfall_data: Vec<ScryfallData>) -> Vec<Card>;
}

#[cfg(feature = "zerver")]
impl SleeveCardProfile for Vec<CardProfile> {
    fn sleeve(self, scryfall_data: Vec<ScryfallData>) -> Vec<Card> {
        let mut data_map: HashMap<Uuid, ScryfallData> = scryfall_data
            .into_iter()
            .map(|sfd| (sfd.id.to_owned(), sfd))
            .collect();

        self.into_iter()
            .filter_map(|cp| {
                data_map
                    .remove(&cp.scryfall_data_id)
                    .map(|sfd| Card::new(cp, sfd))
            })
            .collect::<Vec<Card>>()
    }
}
