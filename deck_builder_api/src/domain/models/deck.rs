use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::models::types::MtgFormat;

/// Complete deck data as stored in the database
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Deck {
    pub id: i32,
    pub name: String,
    pub format: MtgFormat,
    pub user_id: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
