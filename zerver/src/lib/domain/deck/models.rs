use crate::domain::{card::models::scryfall_card::ScryfallCard, deck::models::deck::Deck};

pub mod deck;
pub mod deck_card;

// ======
//  main
// ======

#[derive(Debug, Clone)]
pub struct DeckWithCards {
    deck: Deck,
    cards: Vec<ScryfallCard>,
}

impl DeckWithCards {
    pub fn new(deck: Deck, cards: Vec<ScryfallCard>) -> Self {
        Self { deck, cards }
    }
}
