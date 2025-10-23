use crate::domain::card::models::scryfall_data::colors::Colors;
#[cfg(feature = "zerver")]
use crate::domain::card::models::{
    card_profile::get_card_profile::GetCardProfileError, scryfall_data::SearchScryfallDataError,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum InvalidSearchCards {
    #[error("must include at least one parameter")]
    MissingParameters,
}

#[cfg(feature = "zerver")]
#[derive(Debug, Error)]
pub enum SearchCardsError {
    #[error(transparent)]
    SearchScryfallDataError(SearchScryfallDataError),
    #[error(transparent)]
    GetCardProfileError(GetCardProfileError),
}

#[cfg(feature = "zerver")]
impl From<SearchScryfallDataError> for SearchCardsError {
    fn from(value: SearchScryfallDataError) -> Self {
        SearchCardsError::SearchScryfallDataError(value)
    }
}

#[cfg(feature = "zerver")]
impl From<GetCardProfileError> for SearchCardsError {
    fn from(value: GetCardProfileError) -> Self {
        SearchCardsError::GetCardProfileError(value)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchCards {
    pub name: Option<String>,
    pub type_line: Option<String>,
    pub set: Option<String>,
    pub rarity: Option<String>,
    pub cmc: Option<f64>,
    pub cmc_range: Option<(f64, f64)>,
    pub power: Option<i32>,
    pub power_range: Option<(i32, i32)>,
    pub toughness: Option<i32>,
    pub toughness_range: Option<(i32, i32)>,
    pub color_identity: Option<Colors>,
    pub color_identity_contains: Option<Colors>,
    pub oracle_text: Option<String>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

impl SearchCards {
    pub fn new(
        name: Option<String>,
        type_line: Option<String>,
        set: Option<String>,
        rarity: Option<String>,
        cmc: Option<f64>,
        cmc_range: Option<(f64, f64)>,
        power: Option<i32>,
        power_range: Option<(i32, i32)>,
        toughness: Option<i32>,
        toughness_range: Option<(i32, i32)>,
        color_identity: Option<Colors>,
        color_identity_contains: Option<Colors>,
        oracle_text: Option<String>,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<Self, InvalidSearchCards> {
        if name.is_none()
            && type_line.is_none()
            && set.is_none()
            && rarity.is_none()
            && cmc.is_none()
            && cmc_range.is_none()
            && power.is_none()
            && power_range.is_none()
            && toughness.is_none()
            && toughness_range.is_none()
            && color_identity.is_none()
            && color_identity_contains.is_none()
            && oracle_text.is_none()
        {
            return Err(InvalidSearchCards::MissingParameters);
        }

        let limit = match limit {
            None => Some(20),
            x => x,
        };

        let offset = match offset {
            None => Some(0),
            x => x,
        };

        Ok(Self {
            name,
            type_line,
            set,
            rarity,
            cmc,
            power,
            power_range,
            toughness,
            toughness_range,
            cmc_range,
            color_identity,
            color_identity_contains,
            oracle_text,
            limit,
            offset,
        })
    }
}
