use crate::domain::deck::models::deck::{
    copy_max::{CopyMax, InvalidCopyMax},
    deck_name::{DeckName, InvalidDeckname},
};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum InvalidCreateDeckProfile {
    #[error(transparent)]
    DeckName(InvalidDeckname),
    #[error(transparent)]
    CopyMax(InvalidCopyMax),
}

impl From<InvalidDeckname> for InvalidCreateDeckProfile {
    fn from(value: InvalidDeckname) -> Self {
        Self::DeckName(value)
    }
}

impl From<InvalidCopyMax> for InvalidCreateDeckProfile {
    fn from(value: InvalidCopyMax) -> Self {
        Self::CopyMax(value)
    }
}

#[cfg(feature = "zerver")]
#[derive(Debug, Error)]
pub enum CreateDeckProfileError {
    #[error("deck with name and user id combination already exists")]
    Duplicate,
    #[error("deck created but database returned invalid object {0}")]
    DeckFromDb(anyhow::Error),
    #[error(transparent)]
    Database(anyhow::Error),
}

#[derive(Debug, Clone)]
pub struct CreateDeckProfile {
    pub name: DeckName,
    pub commander_id: Option<Uuid>,
    pub copy_max: Option<CopyMax>,
    pub user_id: Uuid,
}

impl CreateDeckProfile {
    pub fn new(
        name: &str,
        commander_id: Option<Uuid>,
        copy_max: Option<i32>,
        user_id: Uuid,
    ) -> Result<Self, InvalidCreateDeckProfile> {
        let name = DeckName::new(name)?;
        let copy_max: Option<CopyMax> = copy_max.map(|max| CopyMax::new(max)).transpose()?;
        Ok(Self {
            name,
            commander_id,
            copy_max,
            user_id,
        })
    }
}
