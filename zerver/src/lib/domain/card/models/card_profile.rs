pub mod get_card_profile;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, PartialEq, Deserialize)]
pub struct CardProfile {
    pub id: Uuid,
    pub scryfall_data_id: Uuid,
}
