pub mod scryfall_card;
pub mod sync_metrics;
use crate::domain::DatabaseError;
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
#[error("failed to parse `Uuid`")]
pub struct InvalidUuid(uuid::Error);

impl From<uuid::Error> for InvalidUuid {
    fn from(value: uuid::Error) -> Self {
        InvalidUuid(value)
    }
}

/// for errors encountered while creating cards
#[derive(Debug, Error)]
pub enum CreateCardError {
    #[error("id already exists")]
    UniqueConstraintViolation(anyhow::Error),
    #[error(transparent)]
    Database(DatabaseError),
    #[error("card created but database returned invalid object: {0}")]
    InvalidCardFromDatabase(anyhow::Error),
}

/// for errors encountered while getting cards
#[derive(Debug, Error)]
pub enum GetCardError {
    #[error("card not found")]
    NotFound,
    #[error(transparent)]
    Database(DatabaseError),
    #[error("card found but database returned invalid object: {0}")]
    InvalidCardFromDatabase(anyhow::Error),
}

/// for errors encountered while searching cards
/// - NotFound is not a possible enumeration of this
/// because a search request should just return an empty vec
#[derive(Debug, Error)]
pub enum SearchCardError {
    #[error(transparent)]
    Database(DatabaseError),
    #[error("card found but database returned invalid object: {0}")]
    InvalidCardFromDatabase(anyhow::Error),
}

// =======
//  parts
// =======

/// for collecting search parameters
/// while searching for a card
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
// ======
//  main
// ======

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct CardProfile {
    pub id: Uuid,
    pub created_at: Option<chrono::NaiveDateTime>,
    pub updated_at: Option<chrono::NaiveDateTime>,
    pub scryfall_card_id: Uuid,
}

// also `ScryfallCard` but that has its own file
