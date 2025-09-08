use thiserror::Error;
use uuid::Uuid;

use crate::domain::deck::models::deck::GetDeckProfileError;

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
    DeckId(uuid::Error),
    #[error(transparent)]
    CardProfileId(uuid::Error),
    #[error(transparent)]
    Quantity(InvalidQuantity),
}

impl From<InvalidQuantity> for InvalidCreateDeckCard {
    fn from(value: InvalidQuantity) -> Self {
        Self::Quantity(value)
    }
}

#[derive(Debug, Error)]
pub enum CreateDeckCardError {
    #[error("card and deck combination already exists")]
    Duplicate,
    #[error("deck card created but database returned invalid object {0}")]
    DeckCardFromDb(anyhow::Error),
    #[error(transparent)]
    Database(anyhow::Error),
    #[error(transparent)]
    GetDeckProfileError(GetDeckProfileError),
}

impl From<GetDeckProfileError> for CreateDeckCardError {
    fn from(value: GetDeckProfileError) -> Self {
        Self::GetDeckProfileError(value)
    }
}

#[derive(Debug, Error)]
pub enum InvalidGetDeckCard {
    #[error(transparent)]
    DeckId(uuid::Error),
    #[error(transparent)]
    CardProfileId(uuid::Error),
}

#[derive(Debug, Error)]
pub enum GetDeckCardError {
    #[error("deck card not found")]
    NotFound,
    #[error(transparent)]
    Database(anyhow::Error),
    #[error("deck card found but database returned invalid object: {0}")]
    DeckCardFromDb(anyhow::Error),
}

#[derive(Debug, Error)]
pub enum InvalidUpdateDeckCard {
    #[error(transparent)]
    DeckId(uuid::Error),
    #[error(transparent)]
    CardProfileId(uuid::Error),
    #[error(transparent)]
    UpdateQuantity(InvalidUpdateQuanity),
}

impl From<InvalidUpdateQuanity> for InvalidUpdateDeckCard {
    fn from(value: InvalidUpdateQuanity) -> Self {
        Self::UpdateQuantity(value)
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
    DeckCardFromDb(anyhow::Error),
    #[error(transparent)]
    GetDeckProfileError(GetDeckProfileError),
}

impl From<GetDeckProfileError> for UpdateDeckCardError {
    fn from(value: GetDeckProfileError) -> Self {
        Self::GetDeckProfileError(value)
    }
}

#[derive(Debug, Error)]
pub enum DeleteDeckCardError {
    #[error("deck card not found")]
    NotFound,
    #[error(transparent)]
    Database(anyhow::Error),
    #[error(transparent)]
    GetDeckProfileError(GetDeckProfileError),
}

impl From<GetDeckProfileError> for DeleteDeckCardError {
    fn from(value: GetDeckProfileError) -> Self {
        Self::GetDeckProfileError(value)
    }
}

#[derive(Debug, Error)]
pub enum InvalidDeleteDeckCard {
    #[error(transparent)]
    DeckId(uuid::Error),
    #[error(transparent)]
    CardProfileId(uuid::Error),
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
    pub user_id: Uuid,
}

impl CreateDeckCard {
    pub fn new(
        deck_id: &str,
        card_profile_id: &str,
        quantity: i32,
        user_id: Uuid,
    ) -> Result<Self, InvalidCreateDeckCard> {
        let deck_id = Uuid::try_parse(deck_id).map_err(|e| InvalidCreateDeckCard::DeckId(e))?;
        let card_profile_id = Uuid::try_parse(card_profile_id)
            .map_err(|e| InvalidCreateDeckCard::CardProfileId(e))?;
        let quantity = Quantity::new(quantity)?;

        Ok(Self {
            deck_id,
            card_profile_id,
            quantity,
            user_id,
        })
    }
}

#[derive(Debug, Clone)]
pub struct UpdateDeckCard {
    pub deck_id: Uuid,
    pub card_profile_id: Uuid,
    pub update_quantity: UpdateQuantity,
    pub user_id: Uuid,
}

impl UpdateDeckCard {
    pub fn new(
        deck_id: &str,
        card_profile_id: &str,
        update_quantity: i32,
        user_id: Uuid,
    ) -> Result<Self, InvalidUpdateDeckCard> {
        let deck_id = Uuid::try_parse(deck_id).map_err(|e| InvalidUpdateDeckCard::DeckId(e))?;
        let card_profile_id = Uuid::try_parse(card_profile_id)
            .map_err(|e| InvalidUpdateDeckCard::CardProfileId(e))?;
        let update_quantity = UpdateQuantity::new(update_quantity)?;
        Ok(Self {
            deck_id,
            card_profile_id,
            update_quantity,
            user_id,
        })
    }
}

#[derive(Debug, Clone)]
pub struct DeleteDeckCard {
    pub deck_id: Uuid,
    pub card_profile_id: Uuid,
    pub user_id: Uuid,
}

impl DeleteDeckCard {
    pub fn new(
        deck_id: &str,
        card_profile_id: &str,
        user_id: Uuid,
    ) -> Result<Self, InvalidDeleteDeckCard> {
        let deck_id = Uuid::try_parse(deck_id).map_err(|e| InvalidDeleteDeckCard::DeckId(e))?;
        let card_profile_id = Uuid::try_parse(card_profile_id)
            .map_err(|e| InvalidDeleteDeckCard::CardProfileId(e))?;
        Ok(Self {
            deck_id,
            card_profile_id,
            user_id,
        })
    }
}

// ======
//  main
// ======

#[derive(Debug)]
pub struct DeckCard {
    pub deck_id: Uuid,
    pub card_profile_id: Uuid,
    pub quantity: Quantity,
}
