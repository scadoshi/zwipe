//! Card filtering and search query construction.
//!
//! Provides comprehensive filtering capabilities for MTG cards with builder pattern.
//! Filters are applied using PostgreSQL JSONB operators for efficient querying.
//!
//! # Usage
//!
//! ```rust,ignore
//! use zwipe::domain::card::models::search_card::card_filter::CardFilter;
//!
//! let filter = CardFilter::builder()
//!     .name_contains("Lightning Bolt")
//!     .color_identity(vec!["R"])
//!     .cmc_equals(1)
//!     .limit(20)
//!     .build();
//!
//! let cards = card_service.search_cards(&filter).await?;
//! ```

/// Builder pattern for constructing card filters.
pub mod builder;
/// Card filter validation errors.
pub mod error;
/// Getter methods for accessing filter values.
pub mod getters;
/// Sort order options (name, CMC, rarity, etc.).
pub mod order_by_options;

use crate::domain::card::models::{
    scryfall_data::{colors::Colors, rarity::Rarities},
    search_card::{card_filter::order_by_options::OrderByOptions, card_type::CardType},
};
use serde::{Deserialize, Serialize};

/// Validated card search filter with all search criteria.
///
/// Created via [`CardFilterBuilder`](builder::CardFilterBuilder). Contains all
/// filter criteria (text, mana, combat, flags, pagination) for searching cards.
/// At least one filter criterion must be set (enforced by builder).
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
