use crate::domain::card::models::{card_profile::CardProfile, scryfall_data::ScryfallData};

pub mod card_profile;
pub mod scryfall_data;
pub mod sync_metrics;
// ======
//  main
// ======

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

// =========
//  helpers
// =========

pub trait Sleeve {
    pub fn sleeve(self, card_profiles: Vec<CardProfile>) -> Vec<Card>;
}

impl Sleeve for Vec<ScryfallData> {
    fn sleeve(self, card_profiles: Vec<CardProfile>) -> Vec<Card> {
        let data_map: HashMap<Uuid, ScryfallData> = self
            .into_iter()
            .map(|sfd| (sfd.id.to_owned(), sfd))
            .collect();

        card_profiles
            .into_iter()
            .filter_map(|cp| {
                data_map
                    .get(cp.scryfall_data_id)
                    .map(|sfd| Card::new(cp, sfd));
            })
            .collect::<Vec<Card>>();
    }
}
