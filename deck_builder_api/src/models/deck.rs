use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::models::types::MtgFormat;
use crate::schema::decks;

/// Complete deck data as stored in the database
#[derive(Debug, Clone, Queryable, Selectable, Serialize)]
#[diesel(table_name = decks)]
#[diesel(belongs_to(User, foreign_key = user_id))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Deck {
    pub id: i32,
    pub name: String,
    pub format: MtgFormat,
    pub user_id: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

/// Data required to create a new deck in the database
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = decks)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewDeck {
    pub name: String,
    pub format: MtgFormat,
    pub user_id: i32,
}

/// Partial deck data for updating existing cards
#[derive(Debug, AsChangeset, Deserialize)]
#[diesel(table_name = decks)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct UpdateDeck {
    pub name: Option<String>,
    pub format: Option<MtgFormat>,
    pub user_id: Option<i32>,
}

/// Sanitized deck data for API responses
#[derive(Debug, Serialize)]
pub struct DeckResponse {
    pub id: i32,
    pub name: String,
    pub format: MtgFormat,
    pub user_id: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
