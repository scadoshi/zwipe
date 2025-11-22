use crate::domain::card::models::{
    card_profile::get_card_profile::GetCardProfileError,
    scryfall_data::get_scryfall_data::SearchScryfallDataError,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SearchCardsError {
    #[error(transparent)]
    SearchScryfallDataError(SearchScryfallDataError),
    #[error(transparent)]
    GetCardProfileError(GetCardProfileError),
}

impl From<SearchScryfallDataError> for SearchCardsError {
    fn from(value: SearchScryfallDataError) -> Self {
        SearchCardsError::SearchScryfallDataError(value)
    }
}

impl From<GetCardProfileError> for SearchCardsError {
    fn from(value: GetCardProfileError) -> Self {
        SearchCardsError::GetCardProfileError(value)
    }
}
