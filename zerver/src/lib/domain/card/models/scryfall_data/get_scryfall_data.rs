#[cfg(feature = "zerver")]
use crate::domain::{
    card::models::{card_profile::CardProfile, scryfall_data::ScryfallData},
    deck::models::deck_card::DeckCard,
};
use serde::Deserialize;
#[cfg(feature = "zerver")]
use thiserror::Error;
use uuid::Uuid;

#[cfg(feature = "zerver")]
/// Errors that can occur when retrieving Scryfall data.
#[derive(Debug, Error)]
pub enum GetScryfallDataError {
    /// Scryfall data was not found in database.
    #[error("scryfall data not found")]
    NotFound,
    /// Database query/connection error.
    #[error(transparent)]
    Database(anyhow::Error),
}

#[cfg(feature = "zerver")]
/// Errors that can occur when searching for Scryfall data.
#[derive(Debug, Error)]
pub enum SearchScryfallDataError {
    /// Database query/connection error.
    #[error(transparent)]
    Database(anyhow::Error),
}

/// Request to get Scryfall data by Scryfall UUID.
///
/// Wraps a UUID parsed from a string ID.
#[derive(Debug, Clone, Copy)]
pub struct GetScryfallData(Uuid);

impl GetScryfallData {
    /// Creates a new GetScryfallData request by parsing a UUID string.
    ///
    /// # Errors
    /// Returns `uuid::Error` if the string is not a valid UUID.
    pub fn new(id: &str) -> Result<Self, uuid::Error> {
        Ok(Self(Uuid::try_parse(id)?))
    }

    /// Returns the parsed UUID.
    pub fn id(&self) -> Uuid {
        self.0
    }
}

impl<'de> Deserialize<'de> for GetScryfallData {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let id = String::deserialize(deserializer)?;
        GetScryfallData::new(&id).map_err(serde::de::Error::custom)
    }
}

#[cfg(feature = "zerver")]
impl From<&CardProfile> for GetScryfallData {
    fn from(value: &CardProfile) -> Self {
        Self(value.scryfall_data_id)
    }
}

#[cfg(feature = "zerver")]
/// Collection of Scryfall data UUIDs for batch operations.
///
/// Used for bulk fetching Scryfall data by IDs.
/// Derefs to `&[Uuid]` for direct slice operations.
pub struct ScryfallDataIds(Vec<Uuid>);

#[cfg(feature = "zerver")]
impl std::ops::Deref for ScryfallDataIds {
    type Target = [Uuid];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(feature = "zerver")]
impl From<&[DeckCard]> for ScryfallDataIds {
    fn from(value: &[DeckCard]) -> Self {
        value.iter().map(|x| x.scryfall_data_id).collect()
    }
}

#[cfg(feature = "zerver")]
impl From<&[ScryfallData]> for ScryfallDataIds {
    fn from(value: &[ScryfallData]) -> Self {
        value.iter().map(|x| x.id).collect()
    }
}

#[cfg(feature = "zerver")]
impl FromIterator<Uuid> for ScryfallDataIds {
    fn from_iter<T: IntoIterator<Item = Uuid>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}
