pub mod getters;
pub mod setters;

use crate::domain::card::models::{
    scryfall_data::colors::{Color, Colors},
    search_card::{
        card_filter::{error::InvalidCardFilter, CardFilter},
        card_type::CardType,
    },
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct CardFilterBuilder {
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
}

impl Default for CardFilterBuilder {
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
        }
    }
}

impl CardFilterBuilder {
    // constructors
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_empty(&self) -> bool {
        *self
            == *Self::default()
                .set_limit(self.limit)
                .set_offset(self.offset)
    }

    // text
    pub fn with_name_contains(name_contains: impl Into<String>) -> CardFilterBuilder {
        CardFilterBuilder {
            name_contains: Some(name_contains.into()),
            ..CardFilterBuilder::default()
        }
    }

    pub fn with_oracle_text_contains(oracle_text_contains: impl Into<String>) -> CardFilterBuilder {
        CardFilterBuilder {
            oracle_text_contains: Some(oracle_text_contains.into()),
            ..CardFilterBuilder::default()
        }
    }

    // types
    pub fn with_type_line_contains(type_line_contains: impl Into<String>) -> CardFilterBuilder {
        CardFilterBuilder {
            type_line_contains: Some(type_line_contains.into()),
            ..CardFilterBuilder::default()
        }
    }

    pub fn with_type_line_contains_any<I, S>(type_line_contains_any: I) -> CardFilterBuilder
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        CardFilterBuilder {
            type_line_contains_any: Some(
                type_line_contains_any.into_iter().map(Into::into).collect(),
            ),
            ..CardFilterBuilder::default()
        }
    }

    pub fn with_card_type_contains_any<I>(card_type_contains_any: I) -> CardFilterBuilder
    where
        I: IntoIterator<Item = CardType>,
    {
        CardFilterBuilder {
            card_type_contains_any: Some(card_type_contains_any.into_iter().collect()),
            ..CardFilterBuilder::default()
        }
    }

    // printing
    pub fn with_set_contains(set_contains: impl Into<String>) -> CardFilterBuilder {
        CardFilterBuilder {
            set_contains: Some(set_contains.into()),
            ..CardFilterBuilder::default()
        }
    }

    pub fn with_rarity_contains(rarity_contains: impl Into<String>) -> CardFilterBuilder {
        CardFilterBuilder {
            rarity_contains: Some(rarity_contains.into()),
            ..CardFilterBuilder::default()
        }
    }

    // mana
    pub fn with_cmc_equals(cmc_equals: f64) -> CardFilterBuilder {
        CardFilterBuilder {
            cmc_equals: Some(cmc_equals),
            ..CardFilterBuilder::default()
        }
    }

    pub fn with_cmc_range(cmc_range: (f64, f64)) -> CardFilterBuilder {
        CardFilterBuilder {
            cmc_range: Some(cmc_range),
            ..CardFilterBuilder::default()
        }
    }

    pub fn with_color_identity_equals<I>(color_identity_equals: I) -> CardFilterBuilder
    where
        I: IntoIterator<Item = Color>,
    {
        CardFilterBuilder {
            color_identity_equals: Some(color_identity_equals.into_iter().collect()),
            ..CardFilterBuilder::default()
        }
    }

    pub fn with_color_identity_contains_any<I>(color_identity_contains_any: I) -> CardFilterBuilder
    where
        I: IntoIterator<Item = Color>,
    {
        CardFilterBuilder {
            color_identity_contains_any: Some(color_identity_contains_any.into_iter().collect()),
            ..CardFilterBuilder::default()
        }
    }

    // combat
    pub fn with_power_equals(power_equals: i32) -> CardFilterBuilder {
        CardFilterBuilder {
            power_equals: Some(power_equals),
            ..CardFilterBuilder::default()
        }
    }

    pub fn with_power_range(power_range: (i32, i32)) -> CardFilterBuilder {
        CardFilterBuilder {
            power_range: Some(power_range),
            ..CardFilterBuilder::default()
        }
    }

    pub fn with_toughness_equals(toughness_equals: i32) -> CardFilterBuilder {
        CardFilterBuilder {
            toughness_equals: Some(toughness_equals),
            ..CardFilterBuilder::default()
        }
    }

    pub fn with_toughness_range(toughness_range: (i32, i32)) -> CardFilterBuilder {
        CardFilterBuilder {
            toughness_range: Some(toughness_range),
            ..CardFilterBuilder::default()
        }
    }

    // builder
    pub fn build(&self) -> Result<CardFilter, InvalidCardFilter> {
        if self.is_empty() {
            return Err(InvalidCardFilter::Empty);
        }

        Ok(CardFilter {
            power_equals: self.power_equals,
            power_range: self.power_range,
            toughness_equals: self.toughness_equals,
            toughness_range: self.toughness_range,
            cmc_equals: self.cmc_equals,
            cmc_range: self.cmc_range,
            color_identity_contains_any: self.color_identity_contains_any.clone(),
            color_identity_equals: self.color_identity_equals.clone(),
            rarity_contains: self.rarity_contains.clone(),
            set_contains: self.set_contains.clone(),
            name_contains: self.name_contains.clone(),
            oracle_text_contains: self.oracle_text_contains.clone(),
            type_line_contains: self.type_line_contains.clone(),
            type_line_contains_any: self.type_line_contains_any.clone(),
            card_type_contains_any: self.card_type_contains_any.clone(),
            limit: self.limit,
            offset: self.offset,
        })
    }
}
