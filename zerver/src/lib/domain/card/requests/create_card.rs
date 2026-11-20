//! Create/upsert card operation.
//!
//! Inserts or updates card data (Scryfall data + application metadata).
//! Used during Scryfall sync to populate the card database.

use thiserror::Error;

use crate::domain::card::requests::get_scryfall_data::GetScryfallDataError;

/// Errors that can occur during card creation/upsertion.
#[derive(Debug, Error)]
pub enum CreateCardError {
    /// Card ID already exists (should use update/upsert instead).
    #[error("id already exists")]
    UniqueConstraintViolation(anyhow::Error),
    /// A unique column's value belongs to another printing already.
    #[error("{column} {value} is already held by {holder}")]
    Clash {
        /// The unique column.
        column: String,
        /// The value both printings carry.
        value: String,
        /// The printing holding it, as its name and id, or "another printing" when the lookup
        /// finds none.
        holder: String,
    },
    /// Database operation failed.
    #[error(transparent)]
    Database(anyhow::Error),
    /// Scryfall data inserted but database returned invalid object.
    #[error("scryfall data inserted but database returned invalid object: {0}")]
    ScryfallDataFromDb(anyhow::Error),
    /// Card profile created but database returned invalid object.
    #[error("card profile created but database returned invalid object: {0}")]
    CardProfileFromDb(anyhow::Error),
    /// Error retrieving created Scryfall data.
    #[error(transparent)]
    GetScryfallData(anyhow::Error),
}

impl From<GetScryfallDataError> for CreateCardError {
    fn from(value: GetScryfallDataError) -> Self {
        Self::GetScryfallData(value.into())
    }
}
