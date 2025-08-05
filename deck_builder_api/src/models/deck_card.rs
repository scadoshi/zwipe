use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::schema::deck_cards;

/// Complete deck_card data as stored in the database
#[derive(Debug, Clone, Queryable, Selectable, Serialize)]
#[diesel(table_name  = deck_cards)]
#[diesel(belongs_to(Deck, foreign_key = deck_id))]
#[diesel(belongs_to(Card, foreign_key = card_id))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DeckCard {
    pub id: i32,
    pub deck_id: i32,
    pub card_id: i32,
    pub quantity: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

/// Data required to create a new deck_card in the database
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name  = deck_cards)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewDeckCard {
    pub deck_id: i32,
    pub card_id: i32,
    pub quantity: i32,
}

/// Partial deck_card data for updating existing cards
#[derive(Debug, AsChangeset, Deserialize)]
#[diesel(table_name  = deck_cards)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct UpdateDeckCard {
    pub deck_id: Option<i32>,
    pub card_id: Option<i32>,
    pub quantity: Option<i32>,
}

/// Sanitized deck_card data for API responses
#[derive(Debug, Serialize)]
pub struct DeckCardResponse {
    pub id: i32,
    pub deck_id: i32,
    pub card_id: i32,
    pub quantity: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
