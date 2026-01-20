pub mod builder;
pub mod error;
pub mod getters;
pub mod order_by_options;

use crate::domain::card::models::{
    scryfall_data::{colors::Colors, rarity::Rarities},
    search_card::{card_filter::order_by_options::OrderByOptions, card_type::CardType},
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
    color_identity_within: Option<Colors>,
    color_identity_equals: Option<Colors>,
    // rarity
    rarity_equals_any: Option<Rarities>,
    // set
    set_equals_any: Option<Vec<String>>,
    // artist
    artist_equals_any: Option<Vec<String>>,
    // text
    name_contains: Option<String>,
    oracle_text_contains: Option<String>,
    flavor_text_contains: Option<String>,
    has_flavor_text: Option<bool>,
    // types
    type_line_contains: Option<String>,
    type_line_contains_any: Option<Vec<String>>,
    card_type_contains_any: Option<Vec<CardType>>,
    // flags
    is_valid_commander: Option<bool>,
    is_token: Option<bool>,
    is_playable: Option<bool>,
    digital: Option<bool>,
    oversized: Option<bool>,
    promo: Option<bool>,
    content_warning: Option<bool>,
    language: Option<String>,
    // config
    limit: u32,
    offset: u32,
    order_by: Option<OrderByOptions>,
    ascending: bool,
}
