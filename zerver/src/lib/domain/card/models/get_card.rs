//! Get card operation (complete card data).
//!
//! Retrieves both Scryfall data and application metadata for cards.

#[cfg(feature = "zerver")]
use crate::domain::card::models::{
    card_profile::get_card_profile::GetCardProfileError,
    scryfall_data::get_scryfall_data::GetScryfallDataError,
};
#[cfg(feature = "zerver")]
use thiserror::Error;

/// Errors that can occur when retrieving card data.
///
/// Combines errors from getting Scryfall data and card profile.
#[cfg(feature = "zerver")]
#[derive(Debug, Error)]
pub enum GetCardError {
    /// Error retrieving Scryfall data (card attributes from Scryfall API).
    #[error(transparent)]
    GetScryfallDataError(#[from] GetScryfallDataError),
    /// Error retrieving card profile (application metadata).
    #[error(transparent)]
    GetCardProfileError(#[from] GetCardProfileError),
}


/// Errors when constructing a batch card retrieval request.
#[cfg(feature = "zerver")]
#[derive(Debug, Error)]
pub enum InvalidGetCards {
    /// One or more provided IDs are not valid UUIDs.
    #[error("invalid id: {0}")]
    Uuid(uuid::Error),
    /// No card IDs provided (cannot retrieve zero cards).
    #[error("no ids provided")]
    MissingIds,
}

#[cfg(feature = "zerver")]
impl From<uuid::Error> for InvalidGetCards {
    fn from(value: uuid::Error) -> Self {
        Self::Uuid(value)
    }
}
