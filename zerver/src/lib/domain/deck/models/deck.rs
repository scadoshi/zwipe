use std::fmt::Display;

use serde::Serialize;
use thiserror::Error;
use uuid::Uuid;

use crate::domain::{
    card::models::{card_profile::GetCardProfileError, Card, GetCardError},
    deck::models::deck_card::GetDeckCardError,
};

// ========
//  errors
// ========

#[derive(Debug, Error)]
pub enum InvalidDeckname {
    #[error("deck name must be present")]
    Missing,
}

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

#[derive(Debug, Error)]
pub enum CreateDeckProfileError {
    #[error("deck with name and user id combination already exists")]
    Duplicate,
    #[error("deck created but database returned invalid object {0}")]
    DeckFromDb(anyhow::Error),
    #[error(transparent)]
    Database(anyhow::Error),
}

#[derive(Debug, Error)]
pub enum InvalidGetDeck {
    #[error(transparent)]
    InvalidDeckId(uuid::Error),
}

impl From<uuid::Error> for InvalidGetDeck {
    fn from(value: uuid::Error) -> Self {
        Self::InvalidDeckId(value)
    }
}

#[derive(Debug, Error)]
pub enum GetDeckProfileError {
    #[error("deck profile not found")]
    NotFound,
    #[error(transparent)]
    Database(anyhow::Error),
    #[error("deck profile found but database returned invalid object: {0}")]
    DeckProfileFromDb(anyhow::Error),
}

#[derive(Debug, Error)]
pub enum GetDeckError {
    #[error("deck does not belong to authenticated user")]
    DeckNotOwnedByUser,
    #[error(transparent)]
    GetDeckProfileError(GetDeckProfileError),
    #[error(transparent)]
    GetDeckCardError(GetDeckCardError),
    #[error(transparent)]
    GetCardProfileError(GetCardProfileError),
    #[error(transparent)]
    GetCardError(GetCardError),
}

impl From<GetDeckProfileError> for GetDeckError {
    fn from(value: GetDeckProfileError) -> Self {
        Self::GetDeckProfileError(value)
    }
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

impl From<GetCardError> for GetDeckError {
    fn from(value: GetCardError) -> Self {
        Self::GetCardError(value)
    }
}

#[derive(Debug, Error)]
pub enum InvalidUpdateDeckProfile {
    #[error(transparent)]
    InvalidDeckName(InvalidDeckname),
    #[error(transparent)]
    InvalidDeckId(uuid::Error),
    #[error("must update at least one field")]
    NothingToUpdate,
}

impl From<InvalidDeckname> for InvalidUpdateDeckProfile {
    fn from(value: InvalidDeckname) -> Self {
        Self::InvalidDeckName(value)
    }
}

impl From<uuid::Error> for InvalidUpdateDeckProfile {
    fn from(value: uuid::Error) -> Self {
        Self::InvalidDeckId(value)
    }
}

#[derive(Debug, Error)]
pub enum UpdateDeckProfileError {
    #[error("deck with name and user id combination already exists")]
    Duplicate,
    #[error("deck not found")]
    NotFound,
    #[error(transparent)]
    Database(anyhow::Error),
    #[error("deck updated but database returned invalid object: {0}")]
    DeckFromDb(anyhow::Error),
    #[error(transparent)]
    GetDeckProfileError(GetDeckProfileError),
}

impl From<GetDeckProfileError> for UpdateDeckProfileError {
    fn from(value: GetDeckProfileError) -> Self {
        Self::GetDeckProfileError(value)
    }
}

#[derive(Debug, Error)]
pub enum DeleteDeckError {
    #[error("deck not found")]
    NotFound,
    #[error(transparent)]
    Database(anyhow::Error),
}

// ==========
//  newtypes
// ==========

#[derive(Debug, Clone, PartialEq)]
pub struct DeckName(String);

impl DeckName {
    pub fn new(name: &str) -> Result<Self, InvalidDeckname> {
        if name.is_empty() {
            return Err(InvalidDeckname::Missing);
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

#[derive(Debug, Clone)]
pub struct GetDeck {
    pub deck_id: Uuid,
    pub user_id: Uuid,
}

impl GetDeck {
    pub fn new(deck_id: &str, user_id: &Uuid) -> Result<Self, InvalidGetDeck> {
        let deck_id = Uuid::try_parse(deck_id)?;

        Ok(Self {
            deck_id,
            user_id: user_id.clone(),
        })
    }
}

impl From<&UpdateDeckProfile> for GetDeck {
    fn from(value: &UpdateDeckProfile) -> Self {
        Self {
            deck_id: value.deck_id.clone(),
            user_id: value.user_id.clone(),
        }
    }
}

/// for updating deck profiles.
/// though name is the only field
/// i am still leaving as an `Option<T>`
/// to leave room for future additions
#[derive(Debug, Clone)]
pub struct UpdateDeckProfile {
    pub deck_id: Uuid,
    pub name: Option<DeckName>,
    pub user_id: Uuid,
}

impl UpdateDeckProfile {
    pub fn new(
        deck_id: &str,
        name: Option<&str>,
        user_id: Uuid,
    ) -> Result<Self, InvalidUpdateDeckProfile> {
        if name.is_none() {
            return Err(InvalidUpdateDeckProfile::NothingToUpdate);
        }

        let deck_id = Uuid::try_parse(deck_id)?;

        let name = name.map(|name_str| DeckName::new(name_str)).transpose()?;
        Ok(Self {
            deck_id,
            name,
            user_id,
        })
    }
}

#[derive(Debug, Clone)]
pub struct DeleteDeck(Uuid);

impl DeleteDeck {
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

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct DeckProfile {
    pub id: Uuid,
    pub name: DeckName,
    pub user_id: Uuid,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
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
