//! Get multiple deck profiles operation (list user's decks).

use std::str::FromStr;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

#[expect(missing_docs, reason = "the variant names the id that failed to parse")]
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

/// Lists a user's deck profiles.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetDeckProfiles {
    /// User whose decks to list.
    pub user_id: Uuid,
}

impl GetDeckProfiles {
    /// Creates a request for `user_id`'s decks.
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
