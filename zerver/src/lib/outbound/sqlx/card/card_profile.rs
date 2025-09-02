use sqlx_macros::FromRow;
use thiserror::Error;
use uuid::Uuid;

use crate::domain::card::models::card_profile::{CardProfile, GetCardProfileError};

// ========
//  errors
// ========

#[derive(Debug, Error)]
pub enum ToCardProfileError {
    #[error("invalid card profile id: {0}")]
    InvalidId(uuid::Error),
    #[error("invalid card id: {0}")]
    InvalidCardId(uuid::Error),
}

// ======
//  main
// ======

#[derive(Debug, Clone, FromRow)]
pub struct DatabaseCardProfile {
    pub id: String,
    pub scryfall_data_id: String,
}

impl TryFrom<DatabaseCardProfile> for CardProfile {
    type Error = ToCardProfileError;
    fn try_from(value: DatabaseCardProfile) -> Result<Self, Self::Error> {
        let id = Uuid::try_parse(&value.id).map_err(|e| Self::Error::InvalidId(e))?;
        let scryfall_data_id =
            Uuid::try_parse(&value.scryfall_data_id).map_err(|e| Self::Error::InvalidCardId(e))?;
        Ok(Self {
            id,
            scryfall_data_id,
        })
    }
}

impl From<ToCardProfileError> for GetCardProfileError {
    fn from(value: ToCardProfileError) -> Self {
        Self::InvalidCardProfileFromDatabase(value.into())
    }
}
