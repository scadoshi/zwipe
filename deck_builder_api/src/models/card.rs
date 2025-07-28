use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::schema::cards;

/// Complete card data as stored in the database
#[derive(Debug, Clone, Queryable, Selectable, Serialize)]
#[diesel(table_name = cards)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Card {
    pub id: i32,
    pub name: String,
    pub mana_cost: Option<String>,
    pub card_type: String,
    pub rarity: String,
    pub image_url: String,
    pub oracle_text: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

/// Data required to create a new card in the database
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = cards)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewCard {
    pub name: String,
    pub mana_cost: Option<String>,
    pub card_type: String,
    pub rarity: String,
    pub image_url: String,
    pub oracle_text: Option<String>,
}

/// Partial card data for updating existing cards
#[derive(Debug, AsChangeset, Deserialize)]
#[diesel(table_name = cards)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct UpdateCard {
    pub name: Option<String>,
    pub mana_cost: Option<String>,
    pub card_type: Option<String>,
    pub rarity: Option<String>,
    pub image_url: Option<String>,
    pub oracle_text: Option<String>,
}

/// Sanitized card data for API responses
#[derive(Debug, Serialize)]
pub struct CardResponse {
    pub id: i32,
    pub name: String,
    pub mana_cost: Option<String>,
    pub card_type: String,
    pub rarity: String,
    pub image_url: String,
    pub oracle_text: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
