use thiserror::Error;
use zwipe_core::domain::card::search_card::card_filter::error::InvalidCardCriteria;

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

/// Errors from a card lookup.
#[derive(Debug, Error)]
pub enum LookupCardsError {
    /// The lookup built a query the builder refused, which a checked lookup never does.
    #[error(transparent)]
    Query(#[from] InvalidCardCriteria),
    /// The search failed.
    #[error(transparent)]
    Search(#[from] SearchCardsError),
}
