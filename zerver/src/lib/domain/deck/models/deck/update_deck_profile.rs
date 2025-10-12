use crate::domain::deck::models::deck::deck_name::{DeckName, InvalidDeckname};
#[cfg(feature = "zerver")]
use crate::domain::deck::models::deck::get_deck::GetDeckProfileError;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum InvalidUpdateDeckProfile {
    #[error(transparent)]
    DeckName(InvalidDeckname),
    #[error(transparent)]
    DeckId(uuid::Error),
    #[error("must update at least one field")]
    NoUpdates,
}

impl From<InvalidDeckname> for InvalidUpdateDeckProfile {
    fn from(value: InvalidDeckname) -> Self {
        Self::DeckName(value)
    }
}

impl From<uuid::Error> for InvalidUpdateDeckProfile {
    fn from(value: uuid::Error) -> Self {
        Self::DeckId(value)
    }
}

#[cfg(feature = "zerver")]
#[derive(Debug, Error)]
pub enum UpdateDeckProfileError {
    #[error("deck with name and user id combination already exists")]
    Duplicate,
    #[error("deck not found")]
    NotFound,
    #[error(transparent)]
    Database(anyhow::Error),
    #[error("deck updated but database returned invalid object: {0}")]
    DeckFromDb(anyhow::Error),
    #[error(transparent)]
    GetDeckProfileError(GetDeckProfileError),
    #[error("deck does not belong to requesting user")]
    Forbidden,
}

#[cfg(feature = "zerver")]
impl From<GetDeckProfileError> for UpdateDeckProfileError {
    fn from(value: GetDeckProfileError) -> Self {
        Self::GetDeckProfileError(value)
    }
}

/// for updating deck profiles.
/// though name is the only field
/// i am still leaving as an `Option<T>`
/// to leave room for future additions
#[derive(Debug, Clone)]
pub struct UpdateDeckProfile {
    pub deck_id: Uuid,
    pub name: Option<DeckName>,
    pub user_id: Uuid,
}

impl UpdateDeckProfile {
    pub fn new(
        deck_id: &str,
        name: Option<&str>,
        user_id: Uuid,
    ) -> Result<Self, InvalidUpdateDeckProfile> {
        if name.is_none() {
            return Err(InvalidUpdateDeckProfile::NoUpdates);
        }

        let deck_id = Uuid::try_parse(deck_id)?;

        let name = name.map(|name_str| DeckName::new(name_str)).transpose()?;
        Ok(Self {
            deck_id,
            name,
            user_id,
        })
    }
}
