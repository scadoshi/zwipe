//! Card request bodies, and a card's public numbers: the response of
//! `GET /api/card/{oracle_id}/stats`.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::domain::card::models::card_role::CardRole;
use crate::domain::card::models::lookup::{CardTypeWord, LookupName, MAX_LOOKUP_CHIPS};
use crate::domain::card::models::scryfall_data::colors::Color;
use crate::domain::card::models::search_card::card_filter;

/// The body of `POST /api/card/lookup`, a card search open to signed-out callers.
///
/// It names only what to match: a name and the filter's type, color and role chips, each one
/// optional, the name included (left out, it's the empty name). Limits,
/// offsets, sorts and every other search field are refused as unknown, and the server builds the
/// query itself. Each list holds at most [`MAX_LOOKUP_CHIPS`] values, and a lookup names at least
/// one thing to match. Colors are kept once each, in WUBRG order.
///
/// Unlike other requests, unknown fields are refused: an old server refuses a newer client's
/// body with a 422, so a new field ships on the server before any client sends it.
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
    #[serde(default)]
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
    /// no chip is set and the name has no letter or digit: the search drops punctuation before it
    /// matches ([`card_filter::strip_punctuation`]), so a name of only punctuation matches every
    /// card.
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
        let no_name = card_filter::strip_punctuation(name.as_str()).is_empty();
        if no_name && types.is_empty() && colors.is_empty() && card_roles.is_empty() {
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

/// One card's public numbers, pooled across every user and read live.
///
/// Counts only: each client turns them into rates and copy, and no number has a floor, so a card
/// swiped once carries its one swipe. Every field is `#[serde(default)]`, so a field the server
/// doesn't send reads as `None` and a later field is additive. The server sends `in_decks`,
/// `mvp_decks` and `decks_led` on every answer, zero when there are none, and `swipes` and
/// `commander_picks` only once their pool holds a row for the card. Whether the card can lead a
/// deck isn't decided here: the client holding the card decides whether to show `decks_led`
/// and `commander_picks`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct HttpCardStats {
    /// Decks holding the card on the mainboard, in every format.
    #[serde(default)]
    pub in_decks: Option<u64>,
    /// Decks starring the card as an MVP for at least 3 days, the same vesting the global
    /// signal uses.
    #[serde(default)]
    pub mvp_decks: Option<u64>,
    /// Its Add-stack swipes and removals, from every deck in every format.
    #[serde(default)]
    pub swipes: Option<HttpCardSwipes>,
    /// Decks leading with the card, as commander or partner, by any printing.
    #[serde(default)]
    pub decks_led: Option<u64>,
    /// How often Swipe select's commander pick showed the card and how often it was picked.
    #[serde(default)]
    pub commander_picks: Option<HttpCommanderPicks>,
}

/// A card's pooled Add-stack counts. `added`, `skipped` and `maybed` count the swipes of each
/// kind, `shown` counts every time the card was dealt, and `removed` counts the times it was
/// cut from a deck after being added.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct HttpCardSwipes {
    /// Times the card was dealt on an Add stack.
    pub shown: u64,
    /// Right swipes: added to the deck.
    pub added: u64,
    /// Left swipes: skipped.
    pub skipped: u64,
    /// Up swipes: put on the maybeboard.
    pub maybed: u64,
    /// Times it was removed from a deck.
    pub removed: u64,
}

/// A commander's pooled Swipe select counts.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct HttpCommanderPicks {
    /// Times Swipe select dealt it for a commander slot.
    pub shown: u64,
    /// Times it was picked.
    pub selected: u64,
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
    fn the_name_is_optional_and_a_lookup_still_needs_one_criterion() {
        let types_only = read(json!({ "types": ["Goblin"] })).unwrap();
        assert!(types_only.name().is_empty(), "no name is the empty name");
        assert_eq!(types_only.types().len(), 1);
        assert!(read(json!({ "colors": ["R"] })).is_ok());
        assert!(read(json!({ "card_roles": ["ramp"] })).is_ok());
        let nothing = read(json!({})).unwrap_err();
        assert!(
            nothing.contains("a lookup needs a name, a type, a color or a role"),
            "{nothing}"
        );
    }

    #[test]
    fn a_name_with_no_letter_or_digit_is_no_criterion() {
        for name in ["%%%", "___", "--", "!?"] {
            let refused = read(json!({ "name": name })).unwrap_err();
            assert!(
                refused.contains("a lookup needs a name, a type, a color or a role"),
                "{name}: {refused}"
            );
        }
        assert!(
            read(json!({ "name": "%%%", "types": ["Elf"] })).is_ok(),
            "a chip still makes it a lookup"
        );
        assert!(
            read(json!({ "name": "B.F.M." })).is_ok(),
            "punctuation around letters is a name"
        );
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

    #[test]
    fn missing_fields_read_as_none_so_new_ones_stay_additive() {
        let stats: HttpCardStats = serde_json::from_value(json!({ "in_decks": 3 })).unwrap();
        assert_eq!(
            stats,
            HttpCardStats {
                in_decks: Some(3),
                ..HttpCardStats::default()
            }
        );
        let unknown: HttpCardStats =
            serde_json::from_value(json!({ "in_decks": 1, "a_later_field": 9 })).unwrap();
        assert_eq!(unknown.in_decks, Some(1), "an unknown field is ignored");
    }

    #[test]
    fn stats_round_trip() {
        let stats = HttpCardStats {
            in_decks: Some(2),
            mvp_decks: Some(1),
            swipes: Some(HttpCardSwipes {
                shown: 50,
                added: 30,
                skipped: 15,
                maybed: 5,
                removed: 3,
            }),
            decks_led: Some(0),
            commander_picks: None,
        };
        let back: HttpCardStats =
            serde_json::from_value(serde_json::to_value(stats).unwrap()).unwrap();
        assert_eq!(back, stats);
    }
}
