//! Shared card-matching criteria: "what matches a card".
//!
//! [`CardCriteria`] is the predicate core shared by both search paths:
//! flattened into [`CardQuery`](crate::domain::card::models::search_card::card_filter::query::CardQuery) for the server query,
//! and taken by [`Cards`](crate::domain::card::search_card::cards::Cards)
//! operations for in-memory filtering. It carries **no pagination and no
//! ordering**; those are query concerns.

pub mod getters;
pub mod matches;

pub mod text_cap;

pub use matches::PLAYABLE_LAYOUTS;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
pub use text_cap::MAX_TEXT_CRITERION_CHARS;

use crate::domain::card::scryfall_data::colors::Colors;
use crate::domain::card::scryfall_data::rarity::Rarities;
use crate::domain::card::search_card::card_filter::price_currency::PriceCurrency;
use crate::domain::card::search_card::card_type::CardType;
use crate::domain::deck::Format;

/// Validated card-matching criteria (text, mana, combat, flags, legality).
///
/// Created via [`CardQueryBuilder`](crate::domain::card::models::search_card::card_filter::builder::CardQueryBuilder)
/// (`build_criteria()` for the in-memory path, or flattened inside
/// [`CardQuery`](crate::domain::card::models::search_card::card_filter::query::CardQuery) via `build()` for the server path).
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
    #[serde(default, deserialize_with = "text_cap::text_list")]
    pub(super) produced_mana_contains_any: Option<Vec<String>>,
    #[serde(default, deserialize_with = "text_cap::text_list")]
    pub(super) produced_mana_contains_all: Option<Vec<String>>,
    #[serde(default, deserialize_with = "text_cap::text_list")]
    pub(super) produced_mana_excludes: Option<Vec<String>>,
    // rarity
    pub(super) rarity_equals_any: Option<Rarities>,
    pub(super) rarity_excludes_any: Option<Rarities>,
    // set
    #[serde(default, deserialize_with = "text_cap::text_list")]
    pub(super) set_equals_any: Option<Vec<String>>,
    #[serde(default, deserialize_with = "text_cap::text_list")]
    pub(super) set_excludes_any: Option<Vec<String>>,
    // artist
    #[serde(default, deserialize_with = "text_cap::text_list")]
    pub(super) artist_equals_any: Option<Vec<String>>,
    #[serde(default, deserialize_with = "text_cap::text_list")]
    pub(super) artist_excludes_any: Option<Vec<String>>,
    // text
    #[serde(default, deserialize_with = "text_cap::text")]
    pub(super) name_contains: Option<String>,
    #[serde(default, deserialize_with = "text_cap::text")]
    pub(super) name_not_contains: Option<String>,
    #[serde(default, deserialize_with = "text_cap::text")]
    pub(super) oracle_text_contains: Option<String>,
    #[serde(default, deserialize_with = "text_cap::text")]
    pub(super) oracle_text_not_contains: Option<String>,
    #[serde(default, deserialize_with = "text_cap::text_list")]
    pub(super) oracle_text_contains_any: Option<Vec<String>>,
    #[serde(default, deserialize_with = "text_cap::text_list")]
    pub(super) oracle_text_contains_all: Option<Vec<String>>,
    #[serde(default, deserialize_with = "text_cap::text_list")]
    pub(super) oracle_text_excludes_any: Option<Vec<String>>,
    // keywords
    #[serde(default, deserialize_with = "text_cap::text_list")]
    pub(super) keywords_contains_any: Option<Vec<String>>,
    #[serde(default, deserialize_with = "text_cap::text_list")]
    pub(super) keywords_contains_all: Option<Vec<String>>,
    #[serde(default, deserialize_with = "text_cap::text_list")]
    pub(super) keywords_excludes: Option<Vec<String>>,
    #[serde(default, deserialize_with = "text_cap::text")]
    pub(super) flavor_text_contains: Option<String>,
    #[serde(default, deserialize_with = "text_cap::text")]
    pub(super) flavor_text_not_contains: Option<String>,
    pub(super) has_flavor_text: Option<bool>,
    // types
    #[serde(default, deserialize_with = "text_cap::text")]
    pub(super) type_line_contains: Option<String>,
    #[serde(default, deserialize_with = "text_cap::text")]
    pub(super) type_line_not_contains: Option<String>,
    #[serde(default, deserialize_with = "text_cap::text_list")]
    pub(super) type_line_contains_any: Option<Vec<String>>,
    #[serde(default, deserialize_with = "text_cap::text_list")]
    pub(super) type_line_contains_all: Option<Vec<String>>,
    #[serde(default, deserialize_with = "text_cap::text_list")]
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
    #[serde(default, deserialize_with = "text_cap::text")]
    pub(super) language: Option<String>,
    // legalities
    #[serde(default, deserialize_with = "text_cap::text_list")]
    pub(super) legalities_contains_any: Option<Vec<String>>,
    #[serde(default, deserialize_with = "text_cap::text_list")]
    pub(super) legalities_contains_all: Option<Vec<String>>,
    // commander
    pub(super) is_commander_in_format: Option<Format>,
    // partner/background/spell
    pub(super) is_partner: Option<bool>,
    pub(super) is_background: Option<bool>,
    pub(super) is_signature_spell: Option<bool>,
    // Card roles (the coarse role axis). Wire keys are `card_roles_*`.
    #[serde(default, deserialize_with = "text_cap::text_list")]
    pub(super) card_roles_contains_any: Option<Vec<String>>,
    #[serde(default, deserialize_with = "text_cap::text_list")]
    pub(super) card_roles_contains_all: Option<Vec<String>>,
    #[serde(default, deserialize_with = "text_cap::text_list")]
    pub(super) card_roles_excludes: Option<Vec<String>>,
    // oracle tags (granular functional tags)
    #[serde(default, deserialize_with = "text_cap::text_list")]
    pub(super) oracle_tags_contains_any: Option<Vec<String>>,
    #[serde(default, deserialize_with = "text_cap::text_list")]
    pub(super) oracle_tags_contains_all: Option<Vec<String>>,
    #[serde(default, deserialize_with = "text_cap::text_list")]
    pub(super) oracle_tags_excludes: Option<Vec<String>>,
}

#[cfg(test)]
mod tests {
    use super::CardCriteria;

    #[test]
    fn accepts_card_roles_keys() {
        // The role axis uses the `card_roles_*` wire keys, the sole accepted name.
        let via_new: CardCriteria = serde_json::from_str(
            r#"{"card_roles_contains_any":["ramp"],"card_roles_contains_all":["tutor"],"card_roles_excludes":["burn"]}"#,
        )
        .unwrap();
        assert_eq!(
            via_new.card_roles_contains_any.as_deref(),
            Some(["ramp".to_string()].as_slice())
        );
        assert_eq!(
            via_new.card_roles_contains_all.as_deref(),
            Some(["tutor".to_string()].as_slice())
        );
        assert_eq!(
            via_new.card_roles_excludes.as_deref(),
            Some(["burn".to_string()].as_slice())
        );
    }

    #[test]
    fn caps_text_criteria_at_the_wire() {
        let long = "a".repeat(1_000_000);
        let json = serde_json::json!({
            "name_contains": long,
            "oracle_text_contains_any": [long, "draw a card"],
        });
        let criteria: CardCriteria = serde_json::from_value(json).unwrap();
        assert_eq!(
            criteria.name_contains.as_deref().map(str::len),
            Some(super::MAX_TEXT_CRITERION_CHARS)
        );
        let phrases = criteria.oracle_text_contains_any.unwrap();
        assert_eq!(
            phrases.iter().map(String::len).collect::<Vec<_>>(),
            vec![super::MAX_TEXT_CRITERION_CHARS, "draw a card".len()]
        );
    }

    #[test]
    fn keeps_text_criteria_under_the_cap() {
        let json =
            r#"{"name_contains":"Krenko","type_line_contains":"Goblin","set_equals_any":["m10"]}"#;
        let criteria: CardCriteria = serde_json::from_str(json).unwrap();
        assert_eq!(criteria.name_contains.as_deref(), Some("Krenko"));
        assert_eq!(criteria.type_line_contains.as_deref(), Some("Goblin"));
        assert_eq!(criteria.set_equals_any, Some(vec!["m10".to_string()]));
        assert_eq!(criteria.oracle_text_contains, None);
    }

    #[test]
    fn caps_on_a_char_boundary() {
        let long = "é".repeat(super::MAX_TEXT_CRITERION_CHARS + 10);
        let criteria: CardCriteria =
            serde_json::from_value(serde_json::json!({ "name_contains": long })).unwrap();
        assert_eq!(
            criteria.name_contains.map(|s| s.chars().count()),
            Some(super::MAX_TEXT_CRITERION_CHARS)
        );
    }
}
