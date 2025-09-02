pub mod scryfall_card;
pub mod sync_metrics;
use crate::domain::{deck::models::deck_card::DeckCard, DatabaseError};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

// ========
//  errors
// ========

/// represents when an error is encountered
/// while creating a `Uuid` from string
/// usually with the try_parse function
#[derive(Debug, Error)]
#[error("invalid id: {0}")]
pub struct InvalidUuid(uuid::Error);

impl From<uuid::Error> for InvalidUuid {
    fn from(value: uuid::Error) -> Self {
        InvalidUuid(value)
    }
}

#[derive(Debug, Error)]
pub enum GetCardsRequestError {
    #[error("invalid id: {0}")]
    InvalidUuid(uuid::Error),
    #[error("no ids provided")]
    MissingIds,
}

impl From<uuid::Error> for GetCardsRequestError {
    fn from(value: uuid::Error) -> Self {
        Self::InvalidUuid(value)
    }
}

#[derive(Debug, Error)]
pub enum GetCardProfilesRequestError {
    #[error("invalid id: {0}")]
    InvalidUuid(uuid::Error),
    #[error("no ids provided")]
    MissingIds,
}

impl From<uuid::Error> for GetCardProfilesRequestError {
    fn from(value: uuid::Error) -> Self {
        Self::InvalidUuid(value)
    }
}

/// for errors encountered while creating cards
#[derive(Debug, Error)]
pub enum CreateCardError {
    #[error("id already exists")]
    UniqueConstraintViolation(anyhow::Error),
    #[error(transparent)]
    Database(DatabaseError),
    #[error("scryfall card created but database returned invalid object: {0}")]
    InvalidCardFromDatabase(anyhow::Error),
}

/// for errors encountered while getting cards
#[derive(Debug, Error)]
pub enum GetCardError {
    #[error("card not found")]
    NotFound,
    #[error(transparent)]
    Database(DatabaseError),
    #[error("scryfall card found but database returned invalid object: {0}")]
    InvalidCardFromDatabase(anyhow::Error),
}

/// for errors encountered while searching cards
/// - NotFound is not a possible enumeration of this
/// because a search request should just return an empty vec
#[derive(Debug, Error)]
pub enum SearchCardError {
    #[error(transparent)]
    Database(DatabaseError),
    #[error("scryfall card found but database returned invalid object: {0}")]
    InvalidCardFromDatabase(anyhow::Error),
}

#[derive(Debug, Error)]
pub enum GetCardProfileError {
    #[error("card profile not found")]
    NotFound,
    #[error(transparent)]
    Database(DatabaseError),
    #[error("card profile found but database returned invalid object: {0}")]
    InvalidCardProfileFromDatabase(anyhow::Error),
}

// ==========
//  requests
// ==========

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchCardRequest {
    pub name: Option<String>,
    pub type_line: Option<String>,
    pub set: Option<String>,
    pub rarity: Option<String>,
    pub cmc: Option<f64>,
    pub color_identity: Option<Vec<String>>,
    pub oracle_text: Option<String>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

impl Default for SearchCardRequest {
    fn default() -> Self {
        Self {
            name: None,
            type_line: None,
            set: None,
            rarity: None,
            cmc: None,
            color_identity: None,
            oracle_text: None,
            limit: Some(20), // default page size
            offset: Some(0), // start at beginning
        }
    }
}

impl SearchCardRequest {
    pub fn new(
        name: Option<String>,
        type_line: Option<String>,
        set: Option<String>,
        rarity: Option<String>,
        cmc: Option<f64>,
        color_identity: Option<Vec<String>>,
        oracle_text: Option<String>,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Self {
        Self {
            name,
            type_line,
            set,
            rarity,
            cmc,
            color_identity,
            oracle_text,
            limit,
            offset,
        }
    }

    pub fn has_filters(&self) -> bool {
        self.name.is_some()
            || self.type_line.is_some()
            || self.set.is_some()
            || self.rarity.is_some()
            || self.cmc.is_some()
            || self.color_identity.is_some()
            || self.oracle_text.is_some()
            || self.limit.is_some()
            || self.offset.is_some()
    }
}

#[derive(Debug)]
pub struct GetCardRequest(Uuid);

impl GetCardRequest {
    pub fn new(id: &str) -> Result<Self, InvalidUuid> {
        Ok(Self(Uuid::try_parse(id)?))
    }

    pub fn id(&self) -> &Uuid {
        &self.0
    }
}

impl<'de> Deserialize<'de> for GetCardRequest {
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
        GetCardRequest::new(&id)
            .map_err(|e| serde::de::Error::custom(format!("invalid uuid: {}", e.to_string())))
    }
}

pub struct GetCardsRequest(Vec<Uuid>);

impl GetCardsRequest {
    pub fn new(ids: Vec<&str>) -> Result<Self, GetCardsRequestError> {
        if ids.is_empty() {
            return Err(GetCardsRequestError::MissingIds);
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

impl From<Vec<CardProfile>> for GetCardsRequest {
    fn from(value: Vec<CardProfile>) -> Self {
        let ids: Vec<Uuid> = value.into_iter().map(|x| x.scryfall_card_id).collect();
        Self(ids)
    }
}

pub struct GetCardProfileRequest(Uuid);

impl GetCardProfileRequest {
    pub fn new(id: &str) -> Result<Self, InvalidUuid> {
        Ok(Self(Uuid::try_parse(id)?))
    }

    pub fn id(&self) -> &Uuid {
        &self.0
    }
}

pub struct GetCardProfilesRequest(Vec<Uuid>);

impl GetCardProfilesRequest {
    pub fn new(ids: Vec<&str>) -> Result<Self, GetCardProfilesRequestError> {
        if ids.is_empty() {
            return Err(GetCardProfilesRequestError::MissingIds);
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

impl From<Vec<DeckCard>> for GetCardProfilesRequest {
    fn from(value: Vec<DeckCard>) -> Self {
        let ids: Vec<Uuid> = value.into_iter().map(|x| x.card_profile_id).collect();
        Self(ids)
    }
}

// ======
//  main
// ======

#[derive(Debug, Clone)]
pub struct CardProfile {
    pub id: Uuid,
    pub scryfall_card_id: Uuid,
}

// also `ScryfallCard` but that has its own file
