use sqlx_macros::FromRow;
use uuid::Uuid;

use crate::domain::card::models::CardProfile;

#[derive(Debug, Clone, FromRow)]
pub struct DatabaseCardProfile {
    id: String,
    scryfall_card_id: String,
}

impl TryFrom<DatabaseCardProfile> for CardProfile {
    type Error = uuid::Error;
    fn try_from(value: DatabaseCardProfile) -> Result<Self, Self::Error> {
        let id = Uuid::try_parse(&value.id)?;
        let scryfall_card_id = Uuid::try_parse(&value.scryfall_card_id)?;
        Ok(Self {
            id,
            scryfall_card_id,
        })
    }
}
