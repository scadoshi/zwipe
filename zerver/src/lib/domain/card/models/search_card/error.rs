use crate::domain::card::models::{
    card_profile::get_card_profile::GetCardProfileError,
    scryfall_data::get_scryfall_data::SearchScryfallDataError,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SearchCardsError {
    #[error(transparent)]
    SearchScryfallDataError(#[from] SearchScryfallDataError),
    #[error(transparent)]
    GetCardProfileError(#[from] GetCardProfileError),
}
