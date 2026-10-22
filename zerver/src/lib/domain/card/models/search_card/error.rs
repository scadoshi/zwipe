use thiserror::Error;

use crate::domain::card::requests::get_card_profile::GetCardProfileError;
use crate::domain::card::requests::get_scryfall_data::SearchScryfallDataError;

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
