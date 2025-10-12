pub mod create_deck_card;
pub mod delete_deck_card;
pub mod get_deck_card;
pub mod quantity;
pub mod update_deck_card;

use crate::domain::deck::models::deck_card::quantity::Quantity;
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct DeckCard {
    pub deck_id: Uuid,
    pub card_profile_id: Uuid,
    pub quantity: Quantity,
}
