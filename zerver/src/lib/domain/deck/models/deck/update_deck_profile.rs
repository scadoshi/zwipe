#[cfg(feature = "zerver")]
use crate::domain::deck::models::deck::get_deck_profile::GetDeckProfileError;
use crate::domain::deck::models::deck::{
    copy_max::{CopyMax, InvalidCopyMax},
    deck_name::{DeckName, InvalidDeckname},
};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum InvalidUpdateDeckProfile {
    #[error(transparent)]
    DeckName(InvalidDeckname),
    #[error(transparent)]
    CopyMax(InvalidCopyMax),
    #[error("must update at least one field")]
    NoUpdates,
}

impl From<InvalidDeckname> for InvalidUpdateDeckProfile {
    fn from(value: InvalidDeckname) -> Self {
        Self::DeckName(value)
    }
}

impl From<InvalidCopyMax> for InvalidUpdateDeckProfile {
    fn from(value: InvalidCopyMax) -> Self {
        Self::CopyMax(value)
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
    pub commander_id: Option<Option<Uuid>>,
    pub copy_max: Option<Option<CopyMax>>,
    pub user_id: Uuid,
}

impl UpdateDeckProfile {
    pub fn new(
        deck_id: Uuid,
        name: Option<&str>,
        commander_id: Option<Option<Uuid>>,
        copy_max: Option<Option<i32>>,
        user_id: Uuid,
    ) -> Result<Self, InvalidUpdateDeckProfile> {
        if name.is_none() && commander_id.is_none() && copy_max.is_none() {
            return Err(InvalidUpdateDeckProfile::NoUpdates);
        }
        let name = name.map(|name_str| DeckName::new(name_str)).transpose()?;
        let copy_max = copy_max
            .map(|update| update.map(|value| CopyMax::new(value)).transpose())
            .transpose()?;

        Ok(Self {
            deck_id,
            name,
            commander_id,
            copy_max,
            user_id,
        })
    }
}
