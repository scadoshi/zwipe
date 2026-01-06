pub mod get_card_profile;

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, PartialEq, Deserialize)]
pub struct CardProfile {
    pub id: Uuid,
    pub scryfall_data_id: Uuid,
    pub is_valid_commander: bool,
    pub is_token: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
