use std::fmt::Display;

use serde::Serialize;
use thiserror::Error;
use uuid::Uuid;

use crate::domain::{
    card::models::{
        card_profile::GetCardProfileError, scryfall_data::GetScryfallDataError, Card, GetCardError,
    },
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
pub enum InvalidCreateDeck {
    #[error(transparent)]
    DeckName(InvalidDeckname),
}

#[derive(Debug, Error)]
pub enum CreateDeckError {
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
    UserId(uuid::Error),
    #[error("identifier must contain something")]
    MissingIdentifier,
}

#[derive(Debug, Error)]
pub enum GetDeckError {
    #[error("deck not found")]
    NotFound,

    #[error(transparent)]
    Database(anyhow::Error),

    #[error("deck card found but database returned invalid object: {0}")]
    DeckCardFromDb(anyhow::Error),
    #[error("card profile found but database returned invalid object: {0}")]
    CardProfileFromDb(anyhow::Error),
    #[error("deck profile found but database returned invalid object: {0}")]
    DeckProfileFromDb(anyhow::Error),
    // #[error("scryfall data found but database returned invalid object: {0}")]
    // InvalidScryfallDataFromDatabase(anyhow::Error),
    #[error(transparent)]
    GetDeckCardError(GetDeckCardError),
    #[error(transparent)]
    GetCardProfileError(GetCardProfileError),
    #[error(transparent)]
    GetScryfallDataError(GetScryfallDataError),
    #[error(transparent)]
    GetCardError(GetCardError),
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
    InvalidId(uuid::Error),
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
    Database(anyhow::Error),
    #[error("deck updated but database returned invalid object: {0}")]
    InvalidDeckFromDatabase(anyhow::Error),
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

#[derive(Debug, Clone)]
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
    pub fn new(name: &str, user_id: Uuid) -> Result<Self, InvalidDeckname> {
        let name = DeckName::new(name)?;
        Ok(Self { name, user_id })
    }
}

#[derive(Debug, Clone)]
pub struct GetDeck {
    pub identifier: String,
    pub user_id: Uuid,
}

impl GetDeck {
    pub fn new(identifier: &str, user_id: &str) -> Result<Self, InvalidGetDeck> {
        if identifier.is_empty() {
            return Err(InvalidGetDeck::MissingIdentifier);
        }

        let user_id = Uuid::try_parse(user_id).map_err(|e| InvalidGetDeck::UserId(e))?;

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
pub struct UpdateDeckProfile {
    pub id: Uuid,
    pub name: Option<DeckName>,
}

impl UpdateDeckProfile {
    pub fn new(id: &str, name_opt: Option<&str>) -> Result<Self, InvalidUpdateDeckProfile> {
        if name_opt.is_none() {
            return Err(InvalidUpdateDeckProfile::NothingToUpdate);
        }
        let id = Uuid::try_parse(id)?;
        let name = name_opt
            .map(|name_str| DeckName::new(name_str))
            .transpose()?;
        Ok(Self { id, name })
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
