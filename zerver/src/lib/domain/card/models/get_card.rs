#[cfg(feature = "zerver")]
use crate::domain::card::models::{
    card_profile::{get_card_profile::GetCardProfileError, CardProfile},
    scryfall_data::GetScryfallDataError,
};
use serde::Deserialize;
#[cfg(feature = "zerver")]
use thiserror::Error;
use uuid::Uuid;

#[cfg(feature = "zerver")]
#[derive(Debug, Error)]
pub enum GetCardError {
    #[error(transparent)]
    GetScryfallDataError(GetScryfallDataError),
    #[error(transparent)]
    GetCardProfileError(GetCardProfileError),
}

#[cfg(feature = "zerver")]
impl From<GetScryfallDataError> for GetCardError {
    fn from(value: GetScryfallDataError) -> Self {
        Self::GetScryfallDataError(value)
    }
}

#[cfg(feature = "zerver")]
impl From<GetCardProfileError> for GetCardError {
    fn from(value: GetCardProfileError) -> Self {
        Self::GetCardProfileError(value)
    }
}

#[cfg(feature = "zerver")]
#[derive(Debug, Error)]
pub enum InvalidGetCards {
    #[error("invalid id: {0}")]
    Uuid(uuid::Error),
    #[error("no ids provided")]
    MissingIds,
}

#[cfg(feature = "zerver")]
impl From<uuid::Error> for InvalidGetCards {
    fn from(value: uuid::Error) -> Self {
        Self::Uuid(value)
    }
}

#[derive(Debug)]
pub struct GetCard(Uuid);

impl GetCard {
    pub fn new(card_profile_id: &str) -> Result<Self, uuid::Error> {
        Ok(Self(Uuid::try_parse(card_profile_id)?))
    }

    pub fn card_profile_id(&self) -> &Uuid {
        &self.0
    }
}

impl<'de> Deserialize<'de> for GetCard {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let id = String::deserialize(deserializer)?;
        GetCard::new(&id).map_err(serde::de::Error::custom)
    }
}

#[cfg(feature = "zerver")]
pub struct GetCards(Vec<Uuid>);

#[cfg(feature = "zerver")]
impl GetCards {
    pub fn new(ids: Vec<&str>) -> Result<Self, InvalidGetCards> {
        use uuid::Uuid;

        if ids.is_empty() {
            return Err(InvalidGetCards::MissingIds);
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
impl From<&[CardProfile]> for GetCards {
    fn from(value: &[CardProfile]) -> Self {
        Self(
            value
                .into_iter()
                .map(|x| x.scryfall_data_id.to_owned())
                .collect(),
        )
    }
}
