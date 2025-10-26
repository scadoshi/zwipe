#[cfg(feature = "zerver")]
use crate::domain::{
    card::models::{card_profile::get_card_profile::GetCardProfileError, get_card::GetCardError},
    deck::models::{
        deck::get_deck_profile::GetDeckProfileError, deck_card::get_deck_card::GetDeckCardError,
    },
};
#[cfg(feature = "zerver")]
use thiserror::Error;

#[cfg(feature = "zerver")]
#[derive(Debug, Error)]
pub enum GetDeckError {
    #[error(transparent)]
    GetDeckProfileError(GetDeckProfileError),
    #[error(transparent)]
    GetDeckCardError(GetDeckCardError),
    #[error(transparent)]
    GetCardProfileError(GetCardProfileError),
    #[error(transparent)]
    GetCardError(GetCardError),
}

#[cfg(feature = "zerver")]
impl From<GetDeckProfileError> for GetDeckError {
    fn from(value: GetDeckProfileError) -> Self {
        Self::GetDeckProfileError(value)
    }
}

#[cfg(feature = "zerver")]
impl From<GetDeckCardError> for GetDeckError {
    fn from(value: GetDeckCardError) -> Self {
        Self::GetDeckCardError(value)
    }
}

#[cfg(feature = "zerver")]
impl From<GetCardProfileError> for GetDeckError {
    fn from(value: GetCardProfileError) -> Self {
        Self::GetCardProfileError(value)
    }
}

#[cfg(feature = "zerver")]
impl From<GetCardError> for GetDeckError {
    fn from(value: GetCardError) -> Self {
        Self::GetCardError(value)
    }
}
