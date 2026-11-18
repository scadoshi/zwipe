//! Card request bodies.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::domain::card::models::card_role::CardRole;
use crate::domain::card::models::lookup::{CardTypeWord, LookupName, MAX_LOOKUP_CHIPS};
use crate::domain::card::models::scryfall_data::colors::Color;

/// The body of `POST /api/card/lookup`, a card search open to signed-out callers.
///
/// It names only what to match: a name and the filter's type, color and role chips. Limits,
/// offsets, sorts and every other search field are refused as unknown, and the server builds the
/// query itself. Each list holds at most [`MAX_LOOKUP_CHIPS`] values, and a lookup names at least
/// one thing to match. Colors are kept once each, in WUBRG order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "RawCardLookup")]
pub struct HttpCardLookup {
    name: LookupName,
    types: Vec<CardTypeWord>,
    colors: Vec<Color>,
    card_roles: Vec<CardRole>,
}

/// The wire shape before the lookup's own checks.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawCardLookup {
    name: LookupName,
    #[serde(default)]
    types: Vec<CardTypeWord>,
    #[serde(default)]
    colors: Vec<Color>,
    #[serde(default)]
    card_roles: Vec<CardRole>,
}

/// A lookup that can't be run: a list past [`MAX_LOOKUP_CHIPS`], or nothing to match.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum InvalidCardLookup {
    /// More types than a lookup takes.
    #[error("at most {MAX_LOOKUP_CHIPS} types")]
    TooManyTypes,
    /// More colors than a lookup takes.
    #[error("at most {MAX_LOOKUP_CHIPS} colors")]
    TooManyColors,
    /// More roles than a lookup takes.
    #[error("at most {MAX_LOOKUP_CHIPS} roles")]
    TooManyRoles,
    /// No name and no chips.
    #[error("a lookup needs a name, a type, a color or a role")]
    Nothing,
}

impl HttpCardLookup {
    /// A lookup for `name` narrowed by the chips.
    ///
    /// # Errors
    ///
    /// [`InvalidCardLookup`] when a list holds more than [`MAX_LOOKUP_CHIPS`] values, or when
    /// the name is empty and no chip is set.
    pub fn try_new(
        name: LookupName,
        types: Vec<CardTypeWord>,
        colors: Vec<Color>,
        card_roles: Vec<CardRole>,
    ) -> Result<Self, InvalidCardLookup> {
        if types.len() > MAX_LOOKUP_CHIPS {
            return Err(InvalidCardLookup::TooManyTypes);
        }
        if colors.len() > MAX_LOOKUP_CHIPS {
            return Err(InvalidCardLookup::TooManyColors);
        }
        if card_roles.len() > MAX_LOOKUP_CHIPS {
            return Err(InvalidCardLookup::TooManyRoles);
        }
        if name.is_empty() && types.is_empty() && colors.is_empty() && card_roles.is_empty() {
            return Err(InvalidCardLookup::Nothing);
        }
        let colors: BTreeSet<Color> = colors.into_iter().collect();
        Ok(Self {
            name,
            types,
            colors: colors.into_iter().collect(),
            card_roles,
        })
    }

    /// The name to match; empty matches every name.
    #[must_use]
    pub fn name(&self) -> &LookupName {
        &self.name
    }

    /// The type words, any of which a card's type line holds.
    #[must_use]
    pub fn types(&self) -> &[CardTypeWord] {
        &self.types
    }

    /// The colors, once each in WUBRG order, which a card's color identity equals.
    #[must_use]
    pub fn colors(&self) -> &[Color] {
        &self.colors
    }

    /// The roles, any of which a card fills.
    #[must_use]
    pub fn card_roles(&self) -> &[CardRole] {
        &self.card_roles
    }
}

impl TryFrom<RawCardLookup> for HttpCardLookup {
    type Error = InvalidCardLookup;

    fn try_from(raw: RawCardLookup) -> Result<Self, Self::Error> {
        Self::try_new(raw.name, raw.types, raw.colors, raw.card_roles)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn read(body: serde_json::Value) -> Result<HttpCardLookup, String> {
        serde_json::from_value(body).map_err(|e| e.to_string())
    }

    #[test]
    fn a_name_alone_is_a_lookup() {
        let lookup = read(json!({ "name": "bolt" })).unwrap();
        assert_eq!(lookup.name().as_str(), "bolt");
        assert!(lookup.types().is_empty() && lookup.colors().is_empty());
    }

    #[test]
    fn chips_read_and_colors_keep_once_in_wubrg_order() {
        let lookup = read(json!({
            "name": "",
            "types": ["Creature", "Goblin"],
            "colors": ["G", "r", "G"],
            "card_roles": ["ramp", "card_advantage"]
        }))
        .unwrap();
        assert_eq!(lookup.colors(), [Color::Red, Color::Green]);
        assert_eq!(
            lookup.card_roles(),
            [CardRole::Ramp, CardRole::CardAdvantage]
        );
        assert_eq!(lookup.types().len(), 2);
    }

    #[test]
    fn a_limit_offset_sort_or_any_other_field_is_refused() {
        for field in [
            "limit",
            "offset",
            "order_by",
            "direction",
            "synergy",
            "oracle_text_contains",
        ] {
            let mut body = json!({ "name": "bolt" });
            body[field] = json!(1000);
            let error = read(body).unwrap_err();
            assert!(error.contains("unknown field"), "{field}: {error}");
        }
    }

    #[test]
    fn each_list_takes_at_most_five() {
        let six = |value: serde_json::Value| vec![value; MAX_LOOKUP_CHIPS + 1];
        for (field, values) in [
            ("types", six(json!("Goblin"))),
            ("colors", six(json!("R"))),
            ("card_roles", six(json!("ramp"))),
        ] {
            let mut body = json!({ "name": "x" });
            body[field] = json!(values);
            assert!(read(body).is_err(), "{field}");
        }
        assert!(read(json!({ "name": "x", "types": vec!["Goblin"; MAX_LOOKUP_CHIPS] })).is_ok());
    }

    #[test]
    fn unknown_values_and_an_empty_lookup_are_refused() {
        for body in [
            json!({ "name": "" }),
            json!({ "name": "  " }),
            json!({}),
            json!({ "name": "x", "colors": ["P"] }),
            json!({ "name": "x", "card_roles": ["teleport"] }),
            json!({ "name": "x", "types": ["Gob;lin"] }),
            json!({ "name": "a".repeat(101) }),
        ] {
            assert!(read(body.clone()).is_err(), "{body}");
        }
    }
}
