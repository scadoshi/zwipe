use thiserror::Error;
use uuid::Uuid;

use crate::domain::DatabaseError;

// ========
//  errors
// ========

#[derive(Debug, Error)]
#[error("must be greater than 0")]
pub struct InvalidQuantity;

#[derive(Debug, Error)]
#[error("must be less or greater than 0")]
pub struct InvalidAddQuanity;

#[derive(Debug, Error)]
pub enum CreateDeckCardRequestError {
    #[error(transparent)]
    InvalidDeckId(uuid::Error),
    #[error(transparent)]
    InvalidCardId(uuid::Error),
    #[error(transparent)]
    InvalidQuantity(InvalidQuantity),
}

impl From<InvalidQuantity> for CreateDeckCardRequestError {
    fn from(value: InvalidQuantity) -> Self {
        Self::InvalidQuantity(value)
    }
}

#[derive(Debug, Error)]
pub enum CreateDeckCardError {
    #[error("card with that id already exists in that deck")]
    Duplicate,
    #[error("deck card created but database returned invalid object {0}")]
    InvalidDeckCardFromDatabase(anyhow::Error),
    #[error(transparent)]
    Database(DatabaseError),
}

#[derive(Debug, Error)]
pub enum GetDeckCardError {
    #[error("deck card not found")]
    NotFound,
    #[error(transparent)]
    Database(DatabaseError),
    #[error("deck card found but database returned invalid object: {0}")]
    InvalidDeckCardFromDatabase(anyhow::Error),
}

#[derive(Debug, Error)]
pub enum UpdateDeckCardRequestError {
    #[error(transparent)]
    InvalidId(uuid::Error),
    #[error(transparent)]
    InvalidAddQuantity(InvalidAddQuanity),
}

impl From<uuid::Error> for UpdateDeckCardRequestError {
    fn from(value: uuid::Error) -> Self {
        Self::InvalidId(value)
    }
}

impl From<InvalidAddQuanity> for UpdateDeckCardRequestError {
    fn from(value: InvalidAddQuanity) -> Self {
        Self::InvalidAddQuantity(value)
    }
}

#[derive(Debug, Error)]
pub enum UpdateDeckCardError {
    #[error("deck card not found")]
    NotFound,
    #[error("resulting quantity must remain greater than 0")]
    InvalidResultingQuantity,
    #[error(transparent)]
    Database(DatabaseError),
    #[error("deck card updated but database returned invalid object: {0}")]
    InvalidDeckCardFromDatabase(anyhow::Error),
}

#[derive(Debug, Error)]
pub enum DeleteDeckCardError {
    #[error("deck card not found")]
    NotFound,
    #[error(transparent)]
    Database(DatabaseError),
}

// ==========
//  newtypes
// ==========

#[derive(Debug, Clone)]
pub struct Quantity(i32);

impl Quantity {
    pub fn new(quantity: i32) -> Result<Self, InvalidQuantity> {
        if quantity < 1 {
            return Err(InvalidQuantity);
        }
        Ok(Self(quantity))
    }

    pub fn quantity(&self) -> i32 {
        self.0
    }
}

#[derive(Debug, Clone)]
pub struct AddQuantity(i32);

impl AddQuantity {
    pub fn new(add_quantity: i32) -> Result<Self, InvalidAddQuanity> {
        if add_quantity == 0 {
            return Err(InvalidAddQuanity);
        }
        Ok(Self(add_quantity))
    }

    pub fn add_quantity(&self) -> i32 {
        self.0
    }
}

// ==========
//  requests
// ==========

#[derive(Debug, Clone)]
pub struct CreateDeckCardRequest {
    pub deck_id: Uuid,
    pub card_profile_id: Uuid,
    pub quantity: Quantity,
}

impl CreateDeckCardRequest {
    pub fn new(
        deck_id: &str,
        card_profile_id: &str,
        quantity: i32,
    ) -> Result<Self, CreateDeckCardRequestError> {
        let deck_id =
            Uuid::try_parse(deck_id).map_err(|e| CreateDeckCardRequestError::InvalidDeckId(e))?;
        let card_profile_id = Uuid::try_parse(card_profile_id)
            .map_err(|e| CreateDeckCardRequestError::InvalidCardId(e))?;
        let quantity = Quantity::new(quantity)?;

        Ok(Self {
            deck_id,
            card_profile_id,
            quantity,
        })
    }
}

#[derive(Debug, Clone)]
pub struct GetDeckCardRequest(String);

impl GetDeckCardRequest {
    pub fn new(identifier: &str) -> Self {
        Self(identifier.to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone)]
pub struct UpdateDeckCardRequest {
    pub id: Uuid,
    pub add_quantity: AddQuantity,
}

impl UpdateDeckCardRequest {
    pub fn new(id: &str, add_quantity: i32) -> Result<Self, UpdateDeckCardRequestError> {
        let id = Uuid::try_parse(id)?;
        let add_quantity = AddQuantity::new(add_quantity)?;
        Ok(Self { id, add_quantity })
    }
}

#[derive(Debug, Clone)]
pub struct DeleteDeckCardRequest(Uuid);

impl DeleteDeckCardRequest {
    pub fn new(id: &str) -> Result<Self, uuid::Error> {
        let id = Uuid::try_parse(id)?;
        Ok(Self(id))
    }

    pub fn id(&self) -> Uuid {
        self.0
    }
}

// ======
//  main
// ======

#[derive(Debug)]
pub struct DeckCard {
    pub id: Uuid,
    pub deck_id: Uuid,
    pub card_profile_id: Uuid,
    pub quantity: Quantity,
}
