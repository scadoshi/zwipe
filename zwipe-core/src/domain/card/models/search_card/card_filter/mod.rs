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
pub mod order_by_option;

use crate::domain::{
    card::{
        scryfall_data::{colors::Colors, rarity::Rarities},
        search_card::{card_filter::order_by_option::OrderByOption, card_type::CardType},
    },
    deck::Format,
};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// Strips punctuation from a string, keeping only alphanumeric characters and whitespace.
/// Used for punctuation-insensitive text search (e.g., "akromas will" matches "Akroma's Will").
pub fn strip_punctuation(s: &str) -> String {
    s.chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace())
        .collect()
}

/// Validated card search filter with all search criteria.
///
/// Created via [`CardFilterBuilder`](builder::CardFilterBuilder). Contains all
/// filter criteria (text, mana, combat, flags, pagination) for searching cards.
/// At least one filter criterion must be set (enforced by builder).
#[skip_serializing_none]
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
    // produced mana
    produced_mana_contains_any: Option<Vec<String>>,
    produced_mana_contains_all: Option<Vec<String>>,
    produced_mana_excludes: Option<Vec<String>>,
    // rarity
    rarity_equals_any: Option<Rarities>,
    rarity_excludes_any: Option<Rarities>,
    // set
    set_equals_any: Option<Vec<String>>,
    set_excludes_any: Option<Vec<String>>,
    // artist
    artist_equals_any: Option<Vec<String>>,
    artist_excludes_any: Option<Vec<String>>,
    // text
    name_contains: Option<String>,
    name_not_contains: Option<String>,
    oracle_text_contains: Option<String>,
    oracle_text_not_contains: Option<String>,
    oracle_text_contains_any: Option<Vec<String>>,
    oracle_text_contains_all: Option<Vec<String>>,
    oracle_text_excludes_any: Option<Vec<String>>,
    // keywords
    keywords_contains_any: Option<Vec<String>>,
    keywords_contains_all: Option<Vec<String>>,
    keywords_excludes: Option<Vec<String>>,
    flavor_text_contains: Option<String>,
    flavor_text_not_contains: Option<String>,
    has_flavor_text: Option<bool>,
    // types
    type_line_contains: Option<String>,
    type_line_not_contains: Option<String>,
    type_line_contains_any: Option<Vec<String>>,
    type_line_contains_all: Option<Vec<String>>,
    type_line_excludes_any: Option<Vec<String>>,
    card_type_contains_any: Option<Vec<CardType>>,
    card_type_contains_all: Option<Vec<CardType>>,
    card_type_excludes_any: Option<Vec<CardType>>,
    // flags
    is_token: Option<bool>,
    is_playable: Option<bool>,
    digital: Option<bool>,
    oversized: Option<bool>,
    promo: Option<bool>,
    content_warning: Option<bool>,
    language: Option<String>,
    // legalities
    legalities_contains_any: Option<Vec<String>>,
    // commander
    is_commander_in_format: Option<Format>,
    // partner/background/spell
    is_partner: Option<bool>,
    is_background: Option<bool>,
    is_signature_spell: Option<bool>,
    // mechanical category
    mechanical_categories_contains_any: Option<Vec<String>>,
    mechanical_categories_contains_all: Option<Vec<String>>,
    mechanical_categories_excludes: Option<Vec<String>>,
    // config
    #[serde(default = "default_limit")]
    limit: u32,
    #[serde(default)]
    offset: u32,
    order_by: Option<OrderByOption>,
    #[serde(default = "default_ascending")]
    ascending: bool,
}

fn default_limit() -> u32 {
    100
}

fn default_ascending() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::strip_punctuation;

    #[test]
    fn strips_apostrophes() {
        assert_eq!(strip_punctuation("Akroma's Will"), "Akromas Will");
    }

    #[test]
    fn strips_commas() {
        assert_eq!(
            strip_punctuation("Satya, Aetherflux Genius"),
            "Satya Aetherflux Genius"
        );
    }

    #[test]
    fn strips_multiple_punctuation() {
        assert_eq!(
            strip_punctuation("Ætherize — the card!"),
            "Ætherize  the card"
        );
    }

    #[test]
    fn preserves_alphanumeric_and_spaces() {
        assert_eq!(strip_punctuation("Lightning Bolt"), "Lightning Bolt");
    }

    #[test]
    fn empty_string() {
        assert_eq!(strip_punctuation(""), "");
    }

    #[test]
    fn only_punctuation() {
        assert_eq!(strip_punctuation("!@#$%"), "");
    }
}
