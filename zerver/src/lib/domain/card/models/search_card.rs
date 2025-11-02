pub mod card_type;
#[cfg(feature = "zerver")]
use crate::domain::card::models::{
    card_profile::get_card_profile::GetCardProfileError,
    scryfall_data::get_scryfall_data::SearchScryfallDataError,
};
use crate::domain::card::models::{
    scryfall_data::colors::Colors, search_card::card_type::CardType,
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

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct SearchCards {
    pub name_contains: Option<String>,
    pub type_line_contains: Option<String>,
    pub type_line_contains_any: Option<Vec<String>>,
    pub card_type_contains_any: Option<Vec<CardType>>,
    pub set_contains: Option<String>,
    pub rarity_contains: Option<String>,
    pub cmc_equals: Option<f64>,
    pub cmc_range: Option<(f64, f64)>,
    pub power_equals: Option<i32>,
    pub power_range: Option<(i32, i32)>,
    pub toughness_equals: Option<i32>,
    pub toughness_range: Option<(i32, i32)>,
    pub color_identity_equals: Option<Colors>,
    pub color_identity_contains_any: Option<Colors>,
    pub oracle_text_contains: Option<String>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

impl SearchCards {
    pub fn new(
        name_contains: Option<String>,
        type_line_contains: Option<String>,
        type_line_contains_any: Option<Vec<String>>,
        card_type_contains_any: Option<Vec<CardType>>,
        set_contains: Option<String>,
        rarity_contains: Option<String>,
        cmc_equals: Option<f64>,
        cmc_range: Option<(f64, f64)>,
        power_equals: Option<i32>,
        power_range: Option<(i32, i32)>,
        toughness_equals: Option<i32>,
        toughness_range: Option<(i32, i32)>,
        color_identity_equals: Option<Colors>,
        color_identity_contains_any: Option<Colors>,
        oracle_text_contains: Option<String>,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<Self, InvalidSearchCards> {
        if name_contains.is_none()
            && type_line_contains.is_none()
            && type_line_contains_any.is_none()
            && card_type_contains_any.is_none()
            && set_contains.is_none()
            && rarity_contains.is_none()
            && cmc_equals.is_none()
            && cmc_range.is_none()
            && power_equals.is_none()
            && power_range.is_none()
            && toughness_equals.is_none()
            && toughness_range.is_none()
            && color_identity_equals.is_none()
            && color_identity_contains_any.is_none()
            && oracle_text_contains.is_none()
        {
            return Err(InvalidSearchCards::MissingParameters);
        }

        let limit = match limit {
            None => Some(100),
            x => x,
        };

        let offset = match offset {
            None => Some(0),
            x => x,
        };

        Ok(Self {
            name_contains,
            type_line_contains,
            type_line_contains_any,
            card_type_contains_any,
            set_contains,
            rarity_contains,
            cmc_equals,
            cmc_range,
            power_equals,
            power_range,
            toughness_equals,
            toughness_range,
            color_identity_equals,
            color_identity_contains_any,
            oracle_text_contains,
            limit,
            offset,
        })
    }

    pub fn is_blank(&self) -> bool {
        *self == SearchCards::default()
    }

    pub fn by_name(name: &str) -> Self {
        Self {
            name_contains: Some(name.to_string()),
            ..Self::default()
        }
    }
}

impl Default for SearchCards {
    fn default() -> Self {
        Self {
            name_contains: None,
            type_line_contains: None,
            type_line_contains_any: None,
            card_type_contains_any: None,
            set_contains: None,
            rarity_contains: None,
            cmc_equals: None,
            cmc_range: None,
            power_equals: None,
            power_range: None,
            toughness_equals: None,
            toughness_range: None,
            color_identity_equals: None,
            color_identity_contains_any: None,
            oracle_text_contains: None,
            limit: Some(100),
            offset: Some(0),
        }
    }
}
