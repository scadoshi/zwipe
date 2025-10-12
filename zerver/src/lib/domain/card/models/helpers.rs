#[cfg(feature = "zerver")]
use crate::domain::card::models::{card_profile::CardProfile, scryfall_data::ScryfallData, Card};
#[cfg(feature = "zerver")]
use std::collections::HashMap;
#[cfg(feature = "zerver")]
use uuid::Uuid;

#[cfg(feature = "zerver")]
pub trait Sleeve {
    fn sleeve(self, card_profiles: Vec<CardProfile>) -> Vec<Card>;
}

#[cfg(feature = "zerver")]
impl Sleeve for Vec<ScryfallData> {
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
