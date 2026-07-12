//! Mechanical category classification for MTG cards.
//!
//! Cards can have multiple categories (e.g. Sol Ring = Ramp,
//! Lightning Bolt = Burn + Removal).

mod oracle_tag_gaps;

pub use oracle_tag_gaps::classify_oracle_tag_gaps;

use serde::{Deserialize, Serialize};
use std::fmt;

/// Strategic role a card can fill in a deck.
///
/// Cards can belong to multiple categories simultaneously.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CardRole {
    Ramp,
    Draw,
    Removal,
    Wipe,
    Counterspell,
    Protection,
    Evasion,
    Finisher,
    Tokens,
    Lifegain,
    Blink,
    Recursion,
    Mill,
    Burn,
    Drain,
    Pump,
    Anthem,
    Counters,
    Copy,
    Sacrifice,
    Stax,
    Untap,
    Tutor,
    GraveyardHate,
    CardAdvantage,
    Energy,
    Aggression,
}

impl CardRole {
    /// All 27 category variants, alphabetical.
    pub fn all() -> &'static [CardRole] {
        &[
            Self::Aggression,
            Self::Anthem,
            Self::Blink,
            Self::Burn,
            Self::CardAdvantage,
            Self::Copy,
            Self::Counterspell,
            Self::Counters,
            Self::Drain,
            Self::Draw,
            Self::Energy,
            Self::Evasion,
            Self::Finisher,
            Self::GraveyardHate,
            Self::Lifegain,
            Self::Mill,
            Self::Protection,
            Self::Pump,
            Self::Ramp,
            Self::Recursion,
            Self::Removal,
            Self::Sacrifice,
            Self::Stax,
            Self::Tokens,
            Self::Tutor,
            Self::Untap,
            Self::Wipe,
        ]
    }

    /// Compact (≤5 char) abbreviation for chart labels.
    pub fn to_short_name(&self) -> &'static str {
        match self {
            Self::Ramp => "ramp",
            Self::Draw => "draw",
            Self::Removal => "remov",
            Self::Wipe => "wipe",
            Self::Counterspell => "cspel",
            Self::Protection => "protc",
            Self::Evasion => "evasn",
            Self::Finisher => "fnshr",
            Self::Tokens => "tokns",
            Self::Lifegain => "lifgn",
            Self::Blink => "blink",
            Self::Recursion => "recur",
            Self::Mill => "mill",
            Self::Burn => "burn",
            Self::Drain => "drain",
            Self::Pump => "pump",
            Self::Anthem => "anthm",
            Self::Counters => "cntrs",
            Self::Copy => "copy",
            Self::Sacrifice => "sacrf",
            Self::Stax => "stax",
            Self::Untap => "untap",
            Self::Tutor => "tutor",
            Self::GraveyardHate => "grvht",
            Self::CardAdvantage => "cadv",
            Self::Energy => "enrgy",
            Self::Aggression => "aggro",
        }
    }

    /// Human-readable display name.
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Ramp => "Ramp",
            Self::Draw => "Draw",
            Self::Removal => "Removal",
            Self::Wipe => "Wipe",
            Self::Counterspell => "Counterspell",
            Self::Protection => "Protection",
            Self::Evasion => "Evasion",
            Self::Finisher => "Finisher",
            Self::Tokens => "Tokens",
            Self::Lifegain => "Lifegain",
            Self::Blink => "Blink",
            Self::Recursion => "Recursion",
            Self::Mill => "Mill",
            Self::Burn => "Burn",
            Self::Drain => "Drain",
            Self::Pump => "Pump",
            Self::Anthem => "Anthem",
            Self::Counters => "Counters",
            Self::Copy => "Copy",
            Self::Sacrifice => "Sacrifice",
            Self::Stax => "Stax",
            Self::Untap => "Untap",
            Self::Tutor => "Tutor",
            Self::GraveyardHate => "Graveyard Hate",
            Self::CardAdvantage => "Card advantage",
            Self::Energy => "Energy",
            Self::Aggression => "Aggression",
        }
    }
}

/// Display as snake_case (matches serde serialization).
impl fmt::Display for CardRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = serde_json::to_string(self).unwrap_or_default();
        // serde_json wraps in quotes, strip them
        write!(f, "{}", s.trim_matches('"'))
    }
}

/// Error when parsing an unrecognized category string.
#[derive(Debug, thiserror::Error)]
#[error("unknown mechanical category: {0}")]
pub struct InvalidCardRole(pub String);

impl TryFrom<&str> for CardRole {
    type Error = InvalidCardRole;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        serde_json::from_str(&format!("\"{value}\""))
            .map_err(|_| InvalidCardRole(value.to_string()))
    }
}

/// Wire view of a card role for the server-delivered role catalog
/// (`GET /api/card/roles`). Lets a client label any role slug it's sent without
/// a compiled enum — see `context/plans/server_driven_catalogs.md` (Part B).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CardRoleView {
    /// snake_case slug (matches the served role values, e.g. `graveyard_hate`).
    pub slug: String,
    /// Human-readable label (e.g. `Graveyard Hate`).
    pub display_name: String,
    /// Compact (<=5 char) label for chart axes.
    pub short_name: String,
}

impl CardRole {
    /// This role as a wire catalog view (slug + labels).
    pub fn to_view(&self) -> CardRoleView {
        CardRoleView {
            slug: self.to_string(),
            display_name: self.display_name().to_string(),
            short_name: self.to_short_name().to_string(),
        }
    }
}

/// Display label for a role **slug**: the curated `display_name` if the slug is a
/// known `CardRole`, otherwise the prettified slug. Lets card-role display work off
/// server-delivered slugs (`CardProfile.card_roles`) — a server-added role still
/// shows a readable label without a client release, while known roles keep their
/// curated labels (e.g. `counters` → "+1/+1 Counters").
pub fn role_label(slug: &str) -> String {
    CardRole::try_from(slug)
        .map(|r| r.display_name().to_string())
        .unwrap_or_else(|_| prettify_role_slug(slug))
}

/// Title-cases a `snake_case`/`kebab-case` slug (`future_role` → "Future Role").
fn prettify_role_slug(slug: &str) -> String {
    slug.split(['_', '-'])
        .filter(|w| !w.is_empty())
        .map(|w| {
            let mut chars = w.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serde_round_trip() {
        for cat in CardRole::all() {
            let json = serde_json::to_string(cat).unwrap();
            let parsed: CardRole = serde_json::from_str(&json).unwrap();
            assert_eq!(*cat, parsed);
        }
    }

    #[test]
    fn display_matches_serde() {
        assert_eq!(CardRole::GraveyardHate.to_string(), "graveyard_hate");
        assert_eq!(CardRole::Ramp.to_string(), "ramp");
        assert_eq!(CardRole::Counterspell.to_string(), "counterspell");
    }

    #[test]
    fn try_from_valid() {
        assert_eq!(
            CardRole::try_from("graveyard_hate").unwrap(),
            CardRole::GraveyardHate
        );
        assert_eq!(CardRole::try_from("ramp").unwrap(), CardRole::Ramp);
    }

    #[test]
    fn try_from_invalid() {
        assert!(CardRole::try_from("not_a_category").is_err());
    }

    #[test]
    fn role_label_known_curated_unknown_prettified() {
        // Known slug -> curated display name (incl. non-title-case ones that a
        // plain prettify would get wrong, e.g. "Card advantage" not "Card Advantage").
        assert_eq!(role_label("card_advantage"), "Card advantage");
        assert_eq!(role_label("graveyard_hate"), "Graveyard Hate");
        // Unknown (server-added) slug -> prettified, no client release needed.
        assert_eq!(role_label("future_role_2099"), "Future Role 2099");
    }

    #[test]
    fn display_name_readable() {
        assert_eq!(CardRole::GraveyardHate.display_name(), "Graveyard Hate");
        assert_eq!(CardRole::Ramp.display_name(), "Ramp");
    }

    #[test]
    fn all_has_27_variants() {
        assert_eq!(CardRole::all().len(), 27);
    }
}
