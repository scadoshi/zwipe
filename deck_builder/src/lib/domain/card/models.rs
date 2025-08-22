pub mod scryfall_card;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

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
#[error("Card not found")]
pub struct CardNotFound;

#[derive(Debug, Error)]
pub enum CreateCardError {
    #[error("Error: {0}")]
    Unknown(anyhow::Error),
    #[error("Database issues: {0}")]
    DatabaseIssues(anyhow::Error),
    #[error("Card created but database returned invalid object: {0}")]
    InvalidCardFromDatabase(anyhow::Error),
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
    pub colors: Option<String>,
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
            colors: None,
            oracle_text: None,
            limit: Some(20), // Default page size
            offset: Some(0), // Start at beginning
        }
    }
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
