pub mod get_card_profile;

use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct CardProfile {
    pub id: Uuid,
    pub scryfall_data_id: Uuid,
}
