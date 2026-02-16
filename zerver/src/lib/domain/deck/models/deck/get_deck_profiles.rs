use std::str::FromStr;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

#[allow(missing_docs)]
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

#[allow(missing_docs)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetDeckProfiles {
    pub user_id: Uuid,
}

#[allow(missing_docs)]
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
