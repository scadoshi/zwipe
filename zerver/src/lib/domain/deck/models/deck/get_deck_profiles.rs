use std::str::FromStr;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum InvalidGetDeckProfiles {
    #[error(transparent)]
    UserId(uuid::Error),
}

impl From<uuid::Error> for InvalidGetDeckProfiles {
    fn from(value: uuid::Error) -> Self {
        Self::UserId(value)
    }
}

#[derive(Debug, Error)]
pub enum GetDeckProfilesError {
    #[error(transparent)]
    Database(anyhow::Error),
    #[error("deck profile found but database returned invalid object: {0}")]
    DeckProfileFromDb(anyhow::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetDeckProfiles {
    pub user_id: Uuid,
}

impl GetDeckProfiles {
    pub fn new(user_id: Uuid) -> Self {
        Self { user_id }
    }
}

impl FromStr for GetDeckProfiles {
    type Err = InvalidGetDeckProfiles;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let user_id = Uuid::try_parse(s)?;
        Ok(Self { user_id })
    }
}
