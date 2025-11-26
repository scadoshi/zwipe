#[cfg(feature = "zerver")]
use crate::domain::{
    card::models::{
        card_profile::CardProfile, get_card::InvalidGetCards, scryfall_data::ScryfallData,
    },
    deck::models::deck_card::DeckCard,
};
use serde::Deserialize;
#[cfg(feature = "zerver")]
use thiserror::Error;
use uuid::Uuid;

#[cfg(feature = "zerver")]
#[derive(Debug, Error)]
pub enum GetScryfallDataError {
    #[error("scryfall data not found")]
    NotFound,
    #[error(transparent)]
    Database(anyhow::Error),
}

#[cfg(feature = "zerver")]
#[derive(Debug, Error)]
pub enum SearchScryfallDataError {
    #[error(transparent)]
    Database(anyhow::Error),
}

#[derive(Debug, Clone, Copy)]
pub struct GetScryfallData(Uuid);

impl GetScryfallData {
    pub fn new(id: &str) -> Result<Self, uuid::Error> {
        Ok(Self(Uuid::try_parse(id)?))
    }

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
pub struct ScryfallDataIds(Vec<Uuid>);

#[cfg(feature = "zerver")]
impl ScryfallDataIds {
    pub fn new(ids: Vec<&str>) -> Result<Self, InvalidGetCards> {
        use uuid::Uuid;

        if ids.is_empty() {
            return Err(InvalidGetCards::MissingIds);
        }
        Ok(Self(
            ids.into_iter()
                .map(Uuid::try_parse)
                .collect::<Result<Vec<Uuid>, uuid::Error>>()?,
        ))
    }

    pub fn ids(&self) -> &Vec<Uuid> {
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
