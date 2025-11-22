pub mod card_type;
#[cfg(feature = "zerver")]
pub mod error;
mod getters;
mod setters;

use crate::domain::card::models::{
    scryfall_data::colors::{Color, Colors},
    search_card::card_type::CardType,
};
#[cfg(feature = "zerver")]
pub use error::SearchCardsError;
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct Empty;
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Full;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct CardFilterWithState<T> {
    // combat
    power_equals: Option<i32>,
    power_range: Option<(i32, i32)>,
    toughness_equals: Option<i32>,
    toughness_range: Option<(i32, i32)>,
    // mana
    cmc_equals: Option<f64>,
    cmc_range: Option<(f64, f64)>,
    color_identity_contains_any: Option<Colors>,
    color_identity_equals: Option<Colors>,
    // printing
    rarity_contains: Option<String>,
    set_contains: Option<String>,
    // text
    name_contains: Option<String>,
    oracle_text_contains: Option<String>,
    // types
    type_line_contains: Option<String>,
    type_line_contains_any: Option<Vec<String>>,
    card_type_contains_any: Option<Vec<CardType>>,
    // config
    limit: u32,
    offset: u32,
    state: PhantomData<T>,
}

pub type CardFilter = CardFilterWithState<Full>;

impl Default for CardFilterWithState<Empty> {
    fn default() -> Self {
        Self {
            // combat
            power_equals: None,
            power_range: None,
            toughness_equals: None,
            toughness_range: None,
            // mana
            cmc_equals: None,
            cmc_range: None,
            color_identity_contains_any: None,
            color_identity_equals: None,
            // printing
            rarity_contains: None,
            set_contains: None,
            // text
            name_contains: None,
            oracle_text_contains: None,
            // types
            type_line_contains: None,
            type_line_contains_any: None,
            card_type_contains_any: None,
            // config
            limit: 100,
            offset: 0,
            state: PhantomData,
        }
    }
}

impl CardFilterWithState<Full> {
    /// create a `CardFilterWithState<Full>` with all fields empty–kept private for internal use only
    fn marked_full_with_empty_fields() -> Self {
        Self {
            // combat
            power_equals: None,
            power_range: None,
            toughness_equals: None,
            toughness_range: None,
            // mana
            cmc_equals: None,
            cmc_range: None,
            color_identity_contains_any: None,
            color_identity_equals: None,
            // printing
            rarity_contains: None,
            set_contains: None,
            // text
            name_contains: None,
            oracle_text_contains: None,
            // types
            type_line_contains: None,
            type_line_contains_any: None,
            card_type_contains_any: None,
            // config
            limit: 100,
            offset: 0,
            state: PhantomData,
        }
    }
}

impl CardFilterWithState<Empty> {
    pub fn new() -> Self {
        Self::default()
    }

    // text
    pub fn with_name_contains(name_contains: impl Into<String>) -> CardFilterWithState<Full> {
        CardFilterWithState {
            name_contains: Some(name_contains.into()),
            ..CardFilterWithState::marked_full_with_empty_fields()
        }
    }

    pub fn with_oracle_text_contains(
        oracle_text_contains: impl Into<String>,
    ) -> CardFilterWithState<Full> {
        CardFilterWithState {
            oracle_text_contains: Some(oracle_text_contains.into()),
            ..CardFilterWithState::marked_full_with_empty_fields()
        }
    }

    // types
    pub fn with_type_line_contains(
        type_line_contains: impl Into<String>,
    ) -> CardFilterWithState<Full> {
        CardFilterWithState {
            type_line_contains: Some(type_line_contains.into()),
            ..CardFilterWithState::marked_full_with_empty_fields()
        }
    }

    pub fn with_type_line_contains_any<I, S>(type_line_contains_any: I) -> CardFilterWithState<Full>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        CardFilterWithState {
            type_line_contains_any: Some(
                type_line_contains_any.into_iter().map(Into::into).collect(),
            ),
            ..CardFilterWithState::marked_full_with_empty_fields()
        }
    }

    pub fn with_card_type_contains_any<I>(card_type_contains_any: I) -> CardFilterWithState<Full>
    where
        I: IntoIterator<Item = CardType>,
    {
        CardFilterWithState {
            card_type_contains_any: Some(card_type_contains_any.into_iter().collect()),
            ..CardFilterWithState::marked_full_with_empty_fields()
        }
    }

    // printing
    pub fn with_set_contains(set_contains: impl Into<String>) -> CardFilterWithState<Full> {
        CardFilterWithState {
            set_contains: Some(set_contains.into()),
            ..CardFilterWithState::marked_full_with_empty_fields()
        }
    }

    pub fn with_rarity_contains(rarity_contains: impl Into<String>) -> CardFilterWithState<Full> {
        CardFilterWithState {
            rarity_contains: Some(rarity_contains.into()),
            ..CardFilterWithState::marked_full_with_empty_fields()
        }
    }

    // mana
    pub fn with_cmc_equals(cmc_equals: f64) -> CardFilterWithState<Full> {
        CardFilterWithState {
            cmc_equals: Some(cmc_equals),
            ..CardFilterWithState::marked_full_with_empty_fields()
        }
    }

    pub fn with_cmc_range(cmc_range: (f64, f64)) -> CardFilterWithState<Full> {
        CardFilterWithState {
            cmc_range: Some(cmc_range),
            ..CardFilterWithState::marked_full_with_empty_fields()
        }
    }

    pub fn with_color_identity_equals<I>(color_identity_equals: I) -> CardFilterWithState<Full>
    where
        I: IntoIterator<Item = Color>,
    {
        CardFilterWithState {
            color_identity_equals: Some(color_identity_equals.into_iter().collect()),
            ..CardFilterWithState::marked_full_with_empty_fields()
        }
    }

    pub fn with_color_identity_contains_any<I>(
        color_identity_contains_any: I,
    ) -> CardFilterWithState<Full>
    where
        I: IntoIterator<Item = Color>,
    {
        CardFilterWithState {
            color_identity_contains_any: Some(color_identity_contains_any.into_iter().collect()),
            ..CardFilterWithState::marked_full_with_empty_fields()
        }
    }

    // combat
    pub fn with_power_equals(power_equals: i32) -> CardFilterWithState<Full> {
        CardFilterWithState {
            power_equals: Some(power_equals),
            ..CardFilterWithState::marked_full_with_empty_fields()
        }
    }

    pub fn with_power_range(power_range: (i32, i32)) -> CardFilterWithState<Full> {
        CardFilterWithState {
            power_range: Some(power_range),
            ..CardFilterWithState::marked_full_with_empty_fields()
        }
    }

    pub fn with_toughness_equals(toughness_equals: i32) -> CardFilterWithState<Full> {
        CardFilterWithState {
            toughness_equals: Some(toughness_equals),
            ..CardFilterWithState::marked_full_with_empty_fields()
        }
    }

    pub fn with_toughness_range(toughness_range: (i32, i32)) -> CardFilterWithState<Full> {
        CardFilterWithState {
            toughness_range: Some(toughness_range),
            ..CardFilterWithState::marked_full_with_empty_fields()
        }
    }
}
