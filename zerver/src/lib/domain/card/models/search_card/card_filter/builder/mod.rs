pub mod getters;
pub mod setters;

use crate::domain::card::models::{
    scryfall_data::{
        colors::{Color, Colors},
        rarity::Rarities,
    },
    search_card::{
        card_filter::{CardFilter, OrderByOptions, error::InvalidCardFilter},
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
    color_identity_within: Option<Colors>,
    color_identity_equals: Option<Colors>,
    // rarity
    rarity_equals_any: Option<Rarities>,
    // set
    set_equals_any: Option<Vec<String>>,
    // text
    name_contains: Option<String>,
    oracle_text_contains: Option<String>,
    flavor_text_contains: Option<String>,
    has_flavor_text: Option<bool>,
    // types
    type_line_contains: Option<String>,
    type_line_contains_any: Option<Vec<String>>,
    card_type_contains_any: Option<Vec<CardType>>,
    // config
    limit: u32,
    offset: u32,
    order_by: Option<OrderByOptions>,
    ascending: bool,
}

impl Default for CardFilterBuilder {
    fn default() -> Self {
        Self {
            power_equals: None,
            power_range: None,
            toughness_equals: None,
            toughness_range: None,
            cmc_equals: None,
            cmc_range: None,
            color_identity_within: None,
            color_identity_equals: None,
            rarity_equals_any: None,
            set_equals_any: None,
            name_contains: None,
            oracle_text_contains: None,
            flavor_text_contains: None,
            has_flavor_text: None,
            type_line_contains: None,
            type_line_contains_any: None,
            card_type_contains_any: None,
            limit: 100,
            offset: 0,
            order_by: None,
            ascending: true,
        }
    }
}

impl CardFilterBuilder {
    // constructors
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_empty(&self) -> bool {
        let default = Self {
            limit: self.limit,
            offset: self.offset,
            order_by: self.order_by,
            ascending: self.ascending,
            ..Self::default()
        };
        *self == default
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

    pub fn with_flavor_text_contains(flavor_text_contains: impl Into<String>) -> Self {
        Self {
            flavor_text_contains: Some(flavor_text_contains.into()),
            ..Self::default()
        }
    }

    pub fn with_has_flavor_text(has_flavor_text: bool) -> Self {
        Self {
            has_flavor_text: Some(has_flavor_text),
            ..Self::default()
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
    pub fn with_set_contains(
        set_equals_any: impl IntoIterator<Item = impl Into<String>>,
    ) -> CardFilterBuilder {
        CardFilterBuilder {
            set_equals_any: Some(set_equals_any.into_iter().map(|x| x.into()).collect()),
            ..CardFilterBuilder::default()
        }
    }

    pub fn with_rarity_equals_any(rarity_equals_any: Rarities) -> CardFilterBuilder {
        let rarity_equals_any = if rarity_equals_any.is_empty() {
            None
        } else {
            Some(rarity_equals_any)
        };
        CardFilterBuilder {
            rarity_equals_any,
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

    pub fn with_color_identity_within<I>(color_identity_within: I) -> CardFilterBuilder
    where
        I: IntoIterator<Item = Color>,
    {
        CardFilterBuilder {
            color_identity_within: Some(color_identity_within.into_iter().collect()),
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

    // config
    pub fn with_order_by(order_by: OrderByOptions) -> CardFilterBuilder {
        CardFilterBuilder {
            order_by: Some(order_by),
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
            color_identity_within: self.color_identity_within.clone(),
            color_identity_equals: self.color_identity_equals.clone(),
            rarity_equals_any: self.rarity_equals_any.clone(),
            set_equals_any: self.set_equals_any.clone(),
            name_contains: self.name_contains.clone(),
            oracle_text_contains: self.oracle_text_contains.clone(),
            flavor_text_contains: self.flavor_text_contains.clone(),
            has_flavor_text: self.has_flavor_text,
            type_line_contains: self.type_line_contains.clone(),
            type_line_contains_any: self.type_line_contains_any.clone(),
            card_type_contains_any: self.card_type_contains_any.clone(),
            limit: self.limit,
            offset: self.offset,
            order_by: self.order_by,
            ascending: self.ascending,
        })
    }
}
