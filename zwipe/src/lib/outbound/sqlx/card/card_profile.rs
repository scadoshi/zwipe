use sqlx_macros::FromRow;
use uuid::Uuid;

use crate::domain::card::models::card_profile::CardProfile;

// ======
//  main
// ======

#[derive(Debug, Clone, FromRow)]
pub struct DatabaseCardProfile {
    pub id: Uuid,
    pub scryfall_data_id: Uuid,
}

impl From<DatabaseCardProfile> for CardProfile {
    fn from(value: DatabaseCardProfile) -> Self {
        Self {
            id: value.id,
            scryfall_data_id: value.scryfall_data_id,
        }
    }
}
