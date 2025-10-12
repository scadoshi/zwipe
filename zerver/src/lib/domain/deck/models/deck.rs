pub mod create_deck_profile;
pub mod deck_name;
pub mod deck_profile;
pub mod delete_deck;
pub mod get_deck;
pub mod update_deck_profile;

use crate::domain::{card::models::Card, deck::models::deck::deck_profile::DeckProfile};
use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Deck {
    deck_profile: DeckProfile,
    cards: Vec<Card>,
}

impl Deck {
    pub fn new(deck_profile: DeckProfile, cards: Vec<Card>) -> Self {
        Self {
            deck_profile,
            cards,
        }
    }
}
