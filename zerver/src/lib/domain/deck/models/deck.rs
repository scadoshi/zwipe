use std::fmt::Display;

use serde::Serialize;
use thiserror::Error;
use uuid::Uuid;

use crate::domain::{
    card::models::{card_profile::GetCardProfileError, scryfall_data::GetScryfallDataError, Card},
    deck::models::deck_card::GetDeckCardError,
    DatabaseError,
};

// ========
//  errors
// ========

#[derive(Debug, Error)]
pub enum DeckNameError {
    #[error("deck name must be present")]
    MissingDeckName,
}

#[derive(Debug, Error)]
pub enum CreateDeckRequestError {
    #[error(transparent)]
    InvalidName(DeckNameError),
}

#[derive(Debug, Error)]
pub enum CreateDeckError {
    #[error("deck with name and user id combination already exists")]
    Duplicate,
    #[error("deck created but database returned invalid object {0}")]
    InvalidDeckFromDatabase(anyhow::Error),
    #[error(transparent)]
    Database(DatabaseError),
}

#[derive(Debug, Error)]
pub enum GetDeckRequestError {
    #[error(transparent)]
    InvalidUserId(uuid::Error),
    #[error("identifier must contain something")]
    MissingIdentifier,
}

#[derive(Debug, Error)]
pub enum GetDeckError {
    #[error("deck not found")]
    NotFound,
    #[error(transparent)]
    Database(DatabaseError),
    #[error("deck found but database returned invalid object: {0}")]
    InvalidDeckFromDatabase(anyhow::Error),
    #[error(transparent)]
    GetDeckCardError(GetDeckCardError),
    #[error(transparent)]
    GetCardProfileError(GetCardProfileError),
    #[error(transparent)]
    GetScryfallDataError(GetScryfallDataError),
}

impl From<GetDeckCardError> for GetDeckError {
    fn from(value: GetDeckCardError) -> Self {
        Self::GetDeckCardError(value)
    }
}

impl From<GetCardProfileError> for GetDeckError {
    fn from(value: GetCardProfileError) -> Self {
        Self::GetCardProfileError(value)
    }
}

impl From<GetScryfallDataError> for GetDeckError {
    fn from(value: GetScryfallDataError) -> Self {
        Self::GetScryfallDataError(value)
    }
}

#[derive(Debug, Error)]
pub enum UpdateDeckProfileRequestError {
    #[error(transparent)]
    InvalidName(DeckNameError),
    #[error(transparent)]
    InvalidId(uuid::Error),
    #[error("must update at least one field")]
    NothingToUpdate,
}

impl From<DeckNameError> for UpdateDeckProfileRequestError {
    fn from(value: DeckNameError) -> Self {
        Self::InvalidName(value)
    }
}

impl From<uuid::Error> for UpdateDeckProfileRequestError {
    fn from(value: uuid::Error) -> Self {
        Self::InvalidId(value)
    }
}

#[derive(Debug, Error)]
pub enum UpdateDeckProfileError {
    #[error("deck with name and user id combination already exists")]
    Duplicate,
    #[error("deck not found")]
    NotFound,
    #[error(transparent)]
    Database(DatabaseError),
    #[error("deck updated but database returned invalid object: {0}")]
    InvalidDeckFromDatabase(anyhow::Error),
}

/// actual errors encountered while deleting a deck
#[derive(Debug, Error)]
pub enum DeleteDeckError {
    #[error("deck not found")]
    NotFound,
    #[error(transparent)]
    Database(DatabaseError),
}

// ==========
//  newtypes
// ==========

#[derive(Debug, Clone)]
pub struct DeckName(String);

impl DeckName {
    pub fn new(name: &str) -> Result<Self, DeckNameError> {
        if name.is_empty() {
            return Err(DeckNameError::MissingDeckName);
        }
        Ok(Self(name.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for DeckName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Serialize for DeckName {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

// ==========
//  requests
// ==========

#[derive(Debug, Clone)]
pub struct CreateDeckRequest {
    pub name: DeckName,
    pub user_id: Uuid,
}

impl CreateDeckRequest {
    pub fn new(name: &str, user_id: Uuid) -> Result<Self, DeckNameError> {
        let name = DeckName::new(name)?;
        Ok(Self { name, user_id })
    }
}

#[derive(Debug, Clone)]
pub struct GetDeckRequest {
    pub identifier: String,
    pub user_id: Uuid,
}

impl GetDeckRequest {
    pub fn new(identifier: &str, user_id: &str) -> Result<Self, GetDeckRequestError> {
        if identifier.is_empty() {
            return Err(GetDeckRequestError::MissingIdentifier);
        }

        let user_id =
            Uuid::try_parse(user_id).map_err(|e| GetDeckRequestError::InvalidUserId(e))?;

        Ok(Self {
            identifier: identifier.to_string(),
            user_id,
        })
    }
}

/// for updating deck profiles.
/// though name is the only field
/// i am still leaving as an `Option<T>`
/// to leave room for future additions
#[derive(Debug, Clone)]
pub struct UpdateDeckProfileRequest {
    pub id: Uuid,
    pub name: Option<DeckName>,
}

impl UpdateDeckProfileRequest {
    pub fn new(id: &str, name_opt: Option<&str>) -> Result<Self, UpdateDeckProfileRequestError> {
        if name_opt.is_none() {
            return Err(UpdateDeckProfileRequestError::NothingToUpdate);
        }
        let id = Uuid::try_parse(id)?;
        let name = name_opt
            .map(|name_str| DeckName::new(name_str))
            .transpose()?;
        Ok(Self { id, name })
    }
}

#[derive(Debug, Clone)]
pub struct DeleteDeckRequest(Uuid);

impl DeleteDeckRequest {
    pub fn new(id: &str) -> Result<Self, uuid::Error> {
        let trimmed = id.trim();
        let id = Uuid::try_parse(trimmed)?;
        Ok(Self(id))
    }

    pub fn id(&self) -> Uuid {
        self.0
    }
}

// ======
//  main
// ======

#[derive(Debug, Clone)]
pub struct DeckProfile {
    pub id: Uuid,
    pub name: DeckName,
    pub user_id: Uuid,
}

#[derive(Debug, Clone)]
pub struct Deck {
    deck_profile: DeckProfile,
    cards: Vec<Card>,
}

impl Deck {
    pub fn new(deck_profile: DeckProfile, cards: Vec<Card>) -> Self {
        Self {
            deck_profile,
            cards,
        }
    }
}
