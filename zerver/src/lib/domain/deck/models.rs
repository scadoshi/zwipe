use crate::domain::{
    card::models::{scryfall_card::ScryfallCard, CardProfile},
    deck::models::deck::Deck,
};

pub mod deck;
pub mod deck_card;

// ======
//  main
// ======

#[derive(Debug, Clone)]
pub struct Card {
    card_profile: CardProfile,
    scryfall_card: ScryfallCard,
}

#[derive(Debug, Clone)]
pub struct DeckWithCards {
    deck: Deck,
    cards: Vec<Card>,
}

impl DeckWithCards {
    pub fn new(deck: Deck, cards: Vec<Card>) -> Self {
        Self { deck, cards }
    }
}
