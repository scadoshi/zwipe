use crate::domain::card::models::card_profile::CardProfile;
use chrono::NaiveDateTime;
use sqlx_macros::FromRow;
use uuid::Uuid;

// ======
//  main
// ======

#[derive(Debug, Clone, FromRow)]
pub struct DatabaseCardProfile {
    pub id: Uuid,
    pub scryfall_data_id: Uuid,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl From<DatabaseCardProfile> for CardProfile {
    fn from(value: DatabaseCardProfile) -> Self {
        Self {
            id: value.id,
            scryfall_data_id: value.scryfall_data_id,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}
