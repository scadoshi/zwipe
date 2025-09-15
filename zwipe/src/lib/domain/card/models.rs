#[cfg(feature = "zerver")]
use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

use crate::domain::card::models::{
    card_profile::CardProfile,
    scryfall_data::{colors::Colors, ScryfallData},
};

#[cfg(feature = "zerver")]
use crate::domain::card::models::{
    card_profile::GetCardProfileError,
    scryfall_data::{GetScryfallDataError, SearchScryfallDataError},
};

pub mod card_profile;
pub mod scryfall_data;
#[cfg(feature = "zerver")]
pub mod sync_metrics;

// =======
//  error
// =======

#[cfg(feature = "zerver")]
#[derive(Debug, Error)]
pub enum CreateCardError {
    #[error("id already exists")]
    UniqueConstraintViolation(anyhow::Error),
    #[error(transparent)]
    Database(anyhow::Error),
    #[error("scryfall data inserted but database returned invalid object: {0}")]
    ScryfallDataFromDb(anyhow::Error),
    #[error("card profile created but database returned invalid object: {0}")]
    CardProfileFromDb(anyhow::Error),
}

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

#[derive(Debug, Error)]
pub enum InvalidSearchCard {
    #[error("must include at least one parameter")]
    MissingParameters,
}

#[cfg(feature = "zerver")]
#[derive(Debug, Error)]
pub enum SearchCardError {
    #[error(transparent)]
    SearchScryfallDataError(SearchScryfallDataError),
    #[error(transparent)]
    GetCardProfileError(GetCardProfileError),
}

#[cfg(feature = "zerver")]
impl From<SearchScryfallDataError> for SearchCardError {
    fn from(value: SearchScryfallDataError) -> Self {
        SearchCardError::SearchScryfallDataError(value)
    }
}

#[cfg(feature = "zerver")]
impl From<GetCardProfileError> for SearchCardError {
    fn from(value: GetCardProfileError) -> Self {
        SearchCardError::GetCardProfileError(value)
    }
}

// ==========
//  requests
// ==========

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
        let id = String::deserialize(deserializer).map_err(|e| {
            serde::de::Error::custom(format!(
                "failed to deserialize into string: {}",
                e.to_string()
            ))
        })?;
        GetCard::new(&id)
            .map_err(|e| serde::de::Error::custom(format!("invalid uuid: {}", e.to_string())))
    }
}

#[cfg(feature = "zerver")]
pub struct GetCards(Vec<Uuid>);

#[cfg(feature = "zerver")]
impl GetCards {
    pub fn new(ids: Vec<&str>) -> Result<Self, InvalidGetCards> {
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

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchCard {
    pub name: Option<String>,
    pub type_line: Option<String>,
    pub set: Option<String>,
    pub rarity: Option<String>,
    pub cmc: Option<f64>,
    pub cmc_range: Option<(f64, f64)>,
    pub power: Option<i32>,
    pub power_range: Option<(i32, i32)>,
    pub toughness: Option<i32>,
    pub toughness_range: Option<(i32, i32)>,
    pub color_identity: Option<Colors>,
    pub color_identity_contains: Option<Colors>,
    pub oracle_text: Option<String>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

impl SearchCard {
    pub fn new(
        name: Option<String>,
        type_line: Option<String>,
        set: Option<String>,
        rarity: Option<String>,
        cmc: Option<f64>,
        cmc_range: Option<(f64, f64)>,
        power: Option<i32>,
        power_range: Option<(i32, i32)>,
        toughness: Option<i32>,
        toughness_range: Option<(i32, i32)>,
        color_identity: Option<Colors>,
        color_identity_contains: Option<Colors>,
        oracle_text: Option<String>,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<Self, InvalidSearchCard> {
        if name.is_none()
            && type_line.is_none()
            && set.is_none()
            && rarity.is_none()
            && cmc.is_none()
            && cmc_range.is_none()
            && power.is_none()
            && power_range.is_none()
            && toughness.is_none()
            && toughness_range.is_none()
            && color_identity.is_none()
            && color_identity_contains.is_none()
            && oracle_text.is_none()
        {
            return Err(InvalidSearchCard::MissingParameters);
        }

        let limit = match limit {
            None => Some(20),
            x => x,
        };

        let offset = match offset {
            None => Some(0),
            x => x,
        };

        Ok(Self {
            name,
            type_line,
            set,
            rarity,
            cmc,
            power,
            power_range,
            toughness,
            toughness_range,
            cmc_range,
            color_identity,
            color_identity_contains,
            oracle_text,
            limit,
            offset,
        })
    }
}

// ======
//  main
// ======

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Card {
    card_profile: CardProfile,
    scryfall_data: ScryfallData,
}

#[cfg(feature = "zerver")]
impl Card {
    pub fn new(card_profile: CardProfile, scryfall_data: ScryfallData) -> Self {
        Self {
            card_profile,
            scryfall_data,
        }
    }
}

// =========
//  helpers
// =========

#[cfg(feature = "zerver")]
pub trait Sleeve {
    fn sleeve(self, card_profiles: Vec<CardProfile>) -> Vec<Card>;
}

#[cfg(feature = "zerver")]
impl Sleeve for Vec<ScryfallData> {
    fn sleeve(self, card_profiles: Vec<CardProfile>) -> Vec<Card> {
        let mut data_map: HashMap<Uuid, ScryfallData> = self
            .into_iter()
            .map(|sfd| (sfd.id.to_owned(), sfd))
            .collect();

        card_profiles
            .into_iter()
            .filter_map(|cp| {
                data_map
                    .remove(&cp.scryfall_data_id)
                    .map(|sfd| Card::new(cp, sfd))
            })
            .collect::<Vec<Card>>()
    }
}
