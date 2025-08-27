use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx_macros::FromRow;

/// Complete deck_card data as stored in the database
#[derive(Debug, Clone, Deserialize, Serialize, FromRow)]
pub struct DeckCard {
    pub id: i32,
    pub deck_id: i32,
    pub card_id: i32,
    pub quantity: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
