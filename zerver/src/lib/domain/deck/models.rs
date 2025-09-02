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
pub struct FullCard {
    card_profile: CardProfile,
    scryfall_card: ScryfallCard,
}

impl FullCard {
    pub fn new(card_profile: CardProfile, scryfall_card: ScryfallCard) -> Self {
        Self {
            card_profile,
            scryfall_card,
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
