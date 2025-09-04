use thiserror::Error;
use uuid::Uuid;

use crate::domain::deck::models::deck::DeckProfile;

// ========
//  errors
// ========

#[derive(Debug, Error)]
#[error("must be greater than 0")]
pub struct InvalidQuantity;

#[derive(Debug, Error)]
#[error("must be less or greater than 0")]
pub struct InvalidUpdateQuanity;

#[derive(Debug, Error)]
pub enum InvalidCreateDeckCard {
    #[error(transparent)]
    InvalidDeckId(uuid::Error),
    #[error(transparent)]
    InvalidCardId(uuid::Error),
    #[error(transparent)]
    InvalidQuantity(InvalidQuantity),
}

impl From<InvalidQuantity> for InvalidCreateDeckCard {
    fn from(value: InvalidQuantity) -> Self {
        Self::InvalidQuantity(value)
    }
}

#[derive(Debug, Error)]
pub enum CreateDeckCardError {
    #[error("card and deck combination already exists")]
    Duplicate,
    #[error("deck card created but database returned invalid object {0}")]
    InvalidDeckCardFromDatabase(anyhow::Error),
    #[error(transparent)]
    Database(anyhow::Error),
}

#[derive(Debug, Error)]
pub enum InvalidGetDeckCards {
    #[error(transparent)]
    InvalidDeckId(uuid::Error),
}

impl From<uuid::Error> for InvalidGetDeckCards {
    fn from(value: uuid::Error) -> Self {
        Self::InvalidDeckId(value)
    }
}

#[derive(Debug, Error)]
pub enum GetDeckCardError {
    #[error("deck card not found")]
    NotFound,
    #[error(transparent)]
    Database(anyhow::Error),
    #[error("deck card found but database returned invalid object: {0}")]
    InvalidDeckCardFromDatabase(anyhow::Error),
}

#[derive(Debug, Error)]
pub enum InvalidUpdateDeckCard {
    #[error(transparent)]
    InvalidId(uuid::Error),
    #[error(transparent)]
    InvalidAddQuantity(InvalidUpdateQuanity),
}

impl From<uuid::Error> for InvalidUpdateDeckCard {
    fn from(value: uuid::Error) -> Self {
        Self::InvalidId(value)
    }
}

impl From<InvalidUpdateQuanity> for InvalidUpdateDeckCard {
    fn from(value: InvalidUpdateQuanity) -> Self {
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
    Database(anyhow::Error),
    #[error("deck card updated but database returned invalid object: {0}")]
    InvalidDeckCardFromDatabase(anyhow::Error),
}

#[derive(Debug, Error)]
pub enum DeleteDeckCardError {
    #[error("deck card not found")]
    NotFound,
    #[error(transparent)]
    Database(anyhow::Error),
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
pub struct UpdateQuantity(i32);

impl UpdateQuantity {
    pub fn new(add_quantity: i32) -> Result<Self, InvalidUpdateQuanity> {
        if add_quantity == 0 {
            return Err(InvalidUpdateQuanity);
        }
        Ok(Self(add_quantity))
    }

    pub fn value(&self) -> i32 {
        self.0
    }
}

// ==========
//  requests
// ==========

#[derive(Debug, Clone)]
pub struct CreateDeckCard {
    pub deck_id: Uuid,
    pub card_profile_id: Uuid,
    pub quantity: Quantity,
}

impl CreateDeckCard {
    pub fn new(
        deck_id: &str,
        card_profile_id: &str,
        quantity: i32,
    ) -> Result<Self, InvalidCreateDeckCard> {
        let deck_id =
            Uuid::try_parse(deck_id).map_err(|e| InvalidCreateDeckCard::InvalidDeckId(e))?;
        let card_profile_id = Uuid::try_parse(card_profile_id)
            .map_err(|e| InvalidCreateDeckCard::InvalidCardId(e))?;
        let quantity = Quantity::new(quantity)?;

        Ok(Self {
            deck_id,
            card_profile_id,
            quantity,
        })
    }
}

#[derive(Debug, Clone)]
pub struct GetDeckCard {
    pub deck_id: Uuid,
}

impl GetDeckCard {
    pub fn new(deck_id: &str) -> Result<Self, InvalidGetDeckCards> {
        let deck_id = Uuid::try_parse(deck_id)?;
        Ok(Self { deck_id })
    }
}

impl From<&DeckProfile> for GetDeckCard {
    fn from(value: &DeckProfile) -> Self {
        GetDeckCard {
            deck_id: value.id.to_owned(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct UpdateDeckCard {
    pub id: Uuid,
    pub update_quantity: UpdateQuantity,
}

impl UpdateDeckCard {
    pub fn new(id: &str, add_quantity: i32) -> Result<Self, InvalidUpdateDeckCard> {
        let id = Uuid::try_parse(id)?;
        let update_quantity = UpdateQuantity::new(add_quantity)?;
        Ok(Self {
            id,
            update_quantity,
        })
    }
}

#[derive(Debug, Clone)]
pub struct DeleteDeckCard(Uuid);

impl DeleteDeckCard {
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
