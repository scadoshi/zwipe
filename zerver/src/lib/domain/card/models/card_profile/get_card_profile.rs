#[cfg(feature = "zerver")]
use crate::domain::card::models::scryfall_data::ScryfallData;
#[cfg(feature = "zerver")]
use thiserror::Error;
#[cfg(feature = "zerver")]
use uuid::Uuid;

// ========
//  errors
// ========

#[cfg(feature = "zerver")]
#[derive(Debug, Error)]
pub enum GetCardProfileError {
    #[error("card profile not found")]
    NotFound,
    #[error(transparent)]
    Database(anyhow::Error),
    #[error("card profile found but database returned invalid object: {0}")]
    CardProfileFromDb(anyhow::Error),
}

#[cfg(feature = "zerver")]
#[derive(Debug, Error)]
pub enum InvalidCardProfileIds {
    #[error("invalid id: {0}")]
    Uuid(uuid::Error),
    #[error("no ids provided")]
    MissingIds,
}

#[cfg(feature = "zerver")]
impl From<uuid::Error> for InvalidCardProfileIds {
    fn from(value: uuid::Error) -> Self {
        Self::Uuid(value)
    }
}

// ==========
//  requests
// ==========

#[cfg(feature = "zerver")]
pub struct GetCardProfile(Uuid);

#[cfg(feature = "zerver")]
impl GetCardProfile {
    pub fn new(id: &str) -> Result<Self, uuid::Error> {
        Ok(Self(Uuid::try_parse(id)?))
    }

    pub fn id(&self) -> Uuid {
        self.0
    }
}

#[cfg(feature = "zerver")]
#[derive(Debug)]
pub struct CardProfileIds(Vec<Uuid>);

#[cfg(feature = "zerver")]
impl std::ops::Deref for CardProfileIds {
    type Target = [Uuid];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(feature = "zerver")]
impl From<&[ScryfallData]> for CardProfileIds {
    fn from(value: &[ScryfallData]) -> Self {
        Self(value.iter().map(|sfd| sfd.id.to_owned()).collect())
    }
}
