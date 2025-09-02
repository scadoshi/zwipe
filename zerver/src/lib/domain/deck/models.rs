use crate::domain::{
    card::models::{card_profile::CardProfile, scryfall_data::ScryfallData},
    deck::models::deck::Deck,
};

pub mod deck;
pub mod deck_card;

// ======
//  main
// ======

#[derive(Debug, Clone)]
pub struct FullCard {
    card_profile: CardProfile,
    scryfall_data: ScryfallData,
}

impl FullCard {
    pub fn new(card_profile: CardProfile, scryfall_data: ScryfallData) -> Self {
        Self {
            card_profile,
            scryfall_data,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DeckWithCards {
    deck: Deck,
    cards: Vec<FullCard>,
}

impl DeckWithCards {
    pub fn new(deck: Deck, cards: Vec<FullCard>) -> Self {
        Self { deck, cards }
    }
}
