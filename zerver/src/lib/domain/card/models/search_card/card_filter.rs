pub mod builder;
pub mod error;
pub mod getters;

use crate::domain::card::models::{
    scryfall_data::colors::Colors, search_card::card_type::CardType,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct CardFilter {
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
