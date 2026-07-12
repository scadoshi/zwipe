//! Shared card-matching criteria — "what matches a card".
//!
//! [`CardCriteria`] is the predicate core shared by both search paths:
//! flattened into [`CardQuery`](super::query::CardQuery) for the server query,
//! and taken by [`Cards`](crate::domain::card::search_card::cards::Cards)
//! operations for in-memory filtering. It carries **no pagination and no
//! ordering** — those are query concerns.

/// Getter methods for accessing criteria values.
pub mod getters;
/// The in-memory predicate: `CardCriteria::matches(&Card)`.
pub mod matches;

pub use matches::PLAYABLE_LAYOUTS;

use crate::domain::{
    card::{
        scryfall_data::{colors::Colors, rarity::Rarities},
        search_card::{card_filter::price_currency::PriceCurrency, card_type::CardType},
    },
    deck::Format,
};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// Validated card-matching criteria (text, mana, combat, flags, legality).
///
/// Created via [`CardQueryBuilder`](super::builder::CardQueryBuilder)
/// (`build_criteria()` for the in-memory path, or flattened inside
/// [`CardQuery`](super::query::CardQuery) via `build()` for the server path).
/// At least one criterion must be set (enforced by the builder).
#[skip_serializing_none]
#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Clone)]
pub struct CardCriteria {
    // combat
    pub(super) power_equals: Option<i32>,
    pub(super) power_range: Option<(i32, i32)>,
    pub(super) toughness_equals: Option<i32>,
    pub(super) toughness_range: Option<(i32, i32)>,
    // mana
    pub(super) cmc_equals: Option<f64>,
    pub(super) cmc_range: Option<(f64, f64)>,
    pub(super) color_identity_within: Option<Colors>,
    pub(super) color_identity_equals: Option<Colors>,
    // price (min/max against the selected currency's price)
    pub(super) price_min: Option<f64>,
    pub(super) price_max: Option<f64>,
    pub(super) price_currency: Option<PriceCurrency>,
    // produced mana
    pub(super) produced_mana_contains_any: Option<Vec<String>>,
    pub(super) produced_mana_contains_all: Option<Vec<String>>,
    pub(super) produced_mana_excludes: Option<Vec<String>>,
    // rarity
    pub(super) rarity_equals_any: Option<Rarities>,
    pub(super) rarity_excludes_any: Option<Rarities>,
    // set
    pub(super) set_equals_any: Option<Vec<String>>,
    pub(super) set_excludes_any: Option<Vec<String>>,
    // artist
    pub(super) artist_equals_any: Option<Vec<String>>,
    pub(super) artist_excludes_any: Option<Vec<String>>,
    // text
    pub(super) name_contains: Option<String>,
    pub(super) name_not_contains: Option<String>,
    pub(super) oracle_text_contains: Option<String>,
    pub(super) oracle_text_not_contains: Option<String>,
    pub(super) oracle_text_contains_any: Option<Vec<String>>,
    pub(super) oracle_text_contains_all: Option<Vec<String>>,
    pub(super) oracle_text_excludes_any: Option<Vec<String>>,
    // keywords
    pub(super) keywords_contains_any: Option<Vec<String>>,
    pub(super) keywords_contains_all: Option<Vec<String>>,
    pub(super) keywords_excludes: Option<Vec<String>>,
    pub(super) flavor_text_contains: Option<String>,
    pub(super) flavor_text_not_contains: Option<String>,
    pub(super) has_flavor_text: Option<bool>,
    // types
    pub(super) type_line_contains: Option<String>,
    pub(super) type_line_not_contains: Option<String>,
    pub(super) type_line_contains_any: Option<Vec<String>>,
    pub(super) type_line_contains_all: Option<Vec<String>>,
    pub(super) type_line_excludes_any: Option<Vec<String>>,
    pub(super) card_type_contains_any: Option<Vec<CardType>>,
    pub(super) card_type_contains_all: Option<Vec<CardType>>,
    pub(super) card_type_excludes_any: Option<Vec<CardType>>,
    // flags
    pub(super) is_token: Option<bool>,
    pub(super) is_playable: Option<bool>,
    pub(super) digital: Option<bool>,
    pub(super) oversized: Option<bool>,
    pub(super) promo: Option<bool>,
    pub(super) content_warning: Option<bool>,
    pub(super) language: Option<String>,
    // legalities
    pub(super) legalities_contains_any: Option<Vec<String>>,
    // commander
    pub(super) is_commander_in_format: Option<Format>,
    // partner/background/spell
    pub(super) is_partner: Option<bool>,
    pub(super) is_background: Option<bool>,
    pub(super) is_signature_spell: Option<bool>,
    // mechanical category (aka card roles — Phase M dual-accept: the server takes
    // either the legacy `mechanical_categories_*` or the canonical `card_roles_*`
    // wire key into the same field; applied identically downstream).
    #[serde(alias = "card_roles_contains_any")]
    pub(super) mechanical_categories_contains_any: Option<Vec<String>>,
    #[serde(alias = "card_roles_contains_all")]
    pub(super) mechanical_categories_contains_all: Option<Vec<String>>,
    #[serde(alias = "card_roles_excludes")]
    pub(super) mechanical_categories_excludes: Option<Vec<String>>,
    // oracle tags (granular functional tags)
    pub(super) oracle_tags_contains_any: Option<Vec<String>>,
    pub(super) oracle_tags_contains_all: Option<Vec<String>>,
    pub(super) oracle_tags_excludes: Option<Vec<String>>,
}

#[cfg(test)]
mod tests {
    use super::CardCriteria;

    #[test]
    fn accepts_card_roles_alias_and_legacy_keys() {
        // Phase M dual-accept: the canonical `card_roles_*` keys deserialize into
        // the same fields as the legacy `mechanical_categories_*` keys.
        let via_new: CardCriteria = serde_json::from_str(
            r#"{"card_roles_contains_any":["ramp"],"card_roles_excludes":["burn"]}"#,
        )
        .unwrap();
        assert_eq!(
            via_new.mechanical_categories_contains_any.as_deref(),
            Some(["ramp".to_string()].as_slice())
        );
        assert_eq!(
            via_new.mechanical_categories_excludes.as_deref(),
            Some(["burn".to_string()].as_slice())
        );

        // Legacy keys still deserialize unchanged.
        let via_legacy: CardCriteria =
            serde_json::from_str(r#"{"mechanical_categories_contains_all":["tutor"]}"#).unwrap();
        assert_eq!(
            via_legacy.mechanical_categories_contains_all.as_deref(),
            Some(["tutor".to_string()].as_slice())
        );
    }
}
