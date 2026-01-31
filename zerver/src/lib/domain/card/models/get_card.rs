#[cfg(feature = "zerver")]
use crate::domain::card::models::{
    card_profile::get_card_profile::GetCardProfileError,
    scryfall_data::get_scryfall_data::GetScryfallDataError,
};
#[cfg(feature = "zerver")]
use thiserror::Error;

#[cfg(feature = "zerver")]
#[derive(Debug, Error)]
pub enum GetCardError {
    #[error(transparent)]
    GetScryfallDataError(#[from] GetScryfallDataError),
    #[error(transparent)]
    GetCardProfileError(#[from] GetCardProfileError),
}

#[cfg(feature = "zerver")]
#[derive(Debug, Error)]
pub enum InvalidGetCards {
    #[error("invalid id: {0}")]
    Uuid(uuid::Error),
    #[error("no ids provided")]
    MissingIds,
}

#[cfg(feature = "zerver")]
impl From<uuid::Error> for InvalidGetCards {
    fn from(value: uuid::Error) -> Self {
        Self::Uuid(value)
    }
}
