use crate::domain::card::requests::{
    get_card_profile::GetCardProfileError, get_scryfall_data::SearchScryfallDataError,
};
use thiserror::Error;

/// Errors that can occur when searching for cards.
#[derive(Debug, Error)]
pub enum SearchCardsError {
    /// Error occurred while searching Scryfall data.
    #[error(transparent)]
    SearchScryfallDataError(#[from] SearchScryfallDataError),
    /// Error occurred while retrieving card profiles.
    #[error(transparent)]
    GetCardProfileError(#[from] GetCardProfileError),
}
