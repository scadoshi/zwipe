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
    GetDeckProfileError(#[from] GetDeckProfileError),
    #[error(transparent)]
    GetDeckCardError(#[from] GetDeckCardError),
    #[error(transparent)]
    GetCardProfileError(#[from] GetCardProfileError),
    #[error(transparent)]
    GetCardError(#[from] GetCardError),
}
