use crate::domain::{deck::models::deck_card::DeckCard, DatabaseError};
use thiserror::Error;
use uuid::Uuid;

// ========
//  errors
// ========

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

pub struct GetCardProfileRequest(Uuid);

impl GetCardProfileRequest {
    pub fn new(id: &str) -> Result<Self, uuid::Error> {
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
    pub scryfall_data_id: Uuid,
}
