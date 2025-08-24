pub mod scryfall_card;
use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

use crate::{
    domain::card::models::scryfall_card::ScryfallCard,
    outbound::sqlx::postgres::IsUniqueConstraintViolation,
};

// ===================================
//              errors
// ===================================

#[derive(Debug, Error)]
#[error("Failed to parse Uuid")]
pub struct InvalidUuid(uuid::Error);

impl From<uuid::Error> for InvalidUuid {
    fn from(value: uuid::Error) -> Self {
        InvalidUuid(value)
    }
}

#[derive(Debug, Error)]
pub enum CreateCardError {
    #[error("Error: {0}")]
    Unknown(anyhow::Error),
    #[error("ID already exists")]
    UniqueConstraintViolation(anyhow::Error),
    #[error("Database issues: {0}")]
    DatabaseIssues(anyhow::Error),
    #[error("Card created but database returned invalid object: {0}")]
    InvalidCardFromDatabase(anyhow::Error),
}

impl From<sqlx::Error> for CreateCardError {
    fn from(value: sqlx::Error) -> Self {
        if value.is_unique_constraint_violation() {
            return CreateCardError::UniqueConstraintViolation(anyhow!("{value}"));
        }

        CreateCardError::DatabaseIssues(anyhow!("{value}"))
    }
}

#[derive(Debug, Error)]
pub enum GetCardError {
    #[error("Card not found")]
    NotFound,
    #[error("Database issues: {0}")]
    DatabaseIssues(anyhow::Error),
    #[error("Card found but database returned invalid object: {0}")]
    InvalidCardFromDatabase(anyhow::Error),
}

impl From<sqlx::Error> for GetCardError {
    fn from(value: sqlx::Error) -> Self {
        match value {
            sqlx::Error::RowNotFound => GetCardError::NotFound,
            e => GetCardError::DatabaseIssues(anyhow!("{e}")),
        }
    }
}

#[derive(Debug, Error)]
pub enum SearchCardError {
    #[error("Database issues: {0}")]
    DatabaseIssues(anyhow::Error),
    #[error("Card found but database returned invalid object: {0}")]
    InvalidCardFromDatabase(anyhow::Error),
}

impl From<sqlx::Error> for SearchCardError {
    fn from(value: sqlx::Error) -> Self {
        SearchCardError::DatabaseIssues(anyhow!("{value}"))
    }
}

// ================================
//            search params
// ================================

#[derive(Debug, Serialize, Deserialize)]
pub struct CardSearchParameters {
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

impl Default for CardSearchParameters {
    fn default() -> Self {
        Self {
            name: None,
            type_line: None,
            set: None,
            rarity: None,
            cmc: None,
            color_identity: None,
            oracle_text: None,
            limit: Some(20), // Default page size
            offset: Some(0), // Start at beginning
        }
    }
}

impl CardSearchParameters {
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

pub struct CardSearchResult {
    results: Vec<ScryfallCard>,
    limit: u32,
    offset: u32,
}

// ================================
//            card syncs
// ================================

pub struct SyncResult {
    pub cards_processed: usize,
    pub cards_inserted: usize,
    pub cards_skipped: usize,
    pub duration: std::time::Duration,
    pub errors: Vec<String>,
}

// ================================
//            main
// ================================

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct CardProfile {
    pub id: Uuid,
    pub created_at: Option<chrono::NaiveDateTime>,
    pub updated_at: Option<chrono::NaiveDateTime>,
    pub scryfall_card_id: Uuid,
}

// also ScryfallCard but that has its own file
