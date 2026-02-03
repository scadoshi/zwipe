//! Create/upsert card operation.
//!
//! Inserts or updates card data (Scryfall data + application metadata).
//! Used during Scryfall sync to populate the card database.

#[cfg(feature = "zerver")]
use crate::domain::card::models::scryfall_data::get_scryfall_data::GetScryfallDataError;
#[cfg(feature = "zerver")]
use thiserror::Error;

/// Errors that can occur during card creation/upsertion.
#[cfg(feature = "zerver")]
#[derive(Debug, Error)]
pub enum CreateCardError {
    /// Card ID already exists (should use update/upsert instead).
    #[error("id already exists")]
    UniqueConstraintViolation(anyhow::Error),
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

#[cfg(feature = "zerver")]
impl From<GetScryfallDataError> for CreateCardError {
    fn from(value: GetScryfallDataError) -> Self {
        Self::GetScryfallData(value.into())
    }
}
