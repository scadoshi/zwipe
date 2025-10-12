use crate::domain::deck::models::deck::deck_name::{DeckName, InvalidDeckname};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum InvalidCreateDeckProfile {
    #[error(transparent)]
    DeckName(InvalidDeckname),
}

impl From<InvalidDeckname> for InvalidCreateDeckProfile {
    fn from(value: InvalidDeckname) -> Self {
        Self::DeckName(value)
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
    pub user_id: Uuid,
}

impl CreateDeckProfile {
    pub fn new(name: &str, user_id: Uuid) -> Result<Self, InvalidCreateDeckProfile> {
        let name = DeckName::new(name)?;
        Ok(Self { name, user_id })
    }
}
