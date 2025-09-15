use crate::domain::{card::models::scryfall_data::ScryfallData, deck::models::deck_card::DeckCard};
use serde::Serialize;
use thiserror::Error;
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
pub enum InvalidGetCardProfile {
    #[error("invalid id: {0}")]
    Uuid(uuid::Error),
    #[error("no ids provided")]
    MissingIds,
}

#[cfg(feature = "zerver")]
impl From<uuid::Error> for InvalidGetCardProfile {
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

    pub fn id(&self) -> &Uuid {
        &self.0
    }
}

#[cfg(feature = "zerver")]
impl From<&ScryfallData> for GetCardProfile {
    fn from(value: &ScryfallData) -> Self {
        GetCardProfile(value.id.clone())
    }
}

#[cfg(feature = "zerver")]
pub struct GetCardProfiles(Vec<Uuid>);

#[cfg(feature = "zerver")]
impl GetCardProfiles {
    pub fn new(ids: Vec<&str>) -> Result<Self, InvalidGetCardProfile> {
        if ids.is_empty() {
            return Err(InvalidGetCardProfile::MissingIds);
        }
        Ok(Self(
            ids.into_iter()
                .map(|x| Uuid::try_parse(x))
                .collect::<Result<Vec<Uuid>, uuid::Error>>()?,
        ))
    }

    pub fn ids(&self) -> &Vec<Uuid> {
        &self.0
    }
}

#[cfg(feature = "zerver")]
impl From<&[DeckCard]> for GetCardProfiles {
    fn from(value: &[DeckCard]) -> Self {
        Self(
            value
                .into_iter()
                .map(|dc| dc.card_profile_id.to_owned())
                .collect(),
        )
    }
}

#[cfg(feature = "zerver")]
impl From<&[ScryfallData]> for GetCardProfiles {
    fn from(value: &[ScryfallData]) -> Self {
        Self(value.into_iter().map(|sfd| sfd.id.to_owned()).collect())
    }
}

// ======
//  main
// ======

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct CardProfile {
    pub id: Uuid,
    pub scryfall_data_id: Uuid,
}
