pub mod create_deck_card;
pub mod delete_deck_card;
pub mod get_deck_card;
pub mod update_deck_card;

use crate::domain::deck::models::deck_card::DeckCard;
use serde::Serialize;

// ===========
//  http types
// ===========

#[derive(Debug, Serialize)]
pub struct HttpDeckCard {
    pub deck_id: String,
    pub card_profile_id: String,
    pub quantity: i32,
}

impl HttpDeckCard {
    pub fn new(deck_id: &str, card_profile_id: &str, quantity: i32) -> Self {
        Self {
            deck_id: deck_id.to_string(),
            card_profile_id: card_profile_id.to_string(),
            quantity,
        }
    }
}

impl From<DeckCard> for HttpDeckCard {
    fn from(value: DeckCard) -> Self {
        Self {
            deck_id: value.deck_id.to_string(),
            card_profile_id: value.card_profile_id.to_string(),
            quantity: value.quantity.quantity(),
        }
    }
}
