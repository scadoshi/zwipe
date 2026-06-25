//! Deck archetype/strategy tags.
//!
//! A deck can be labeled with a small set of these to describe its overall
//! strategy (Aggro, Tokens, Reanimator, …). Distinct from the card-level
//! `MechanicalCategory`: these describe a whole deck's game plan, not a single
//! card's role. The set is curated (from EDHREC themes and Archidekt/Moxfield
//! tags) and fixed so tags stay clean and filterable.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Maximum number of tags a single deck may carry.
pub const MAX_DECK_TAGS: usize = 5;

/// A deck-level archetype or theme tag.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeckTag {
    Aggro,
    Aristocrats,
    Artifacts,
    Blink,
    Burn,
    Combo,
    Control,
    Counters,
    Enchantress,
    Energy,
    Graveyard,
    GroupHug,
    Infect,
    Landfall,
    Lifedrain,
    Lifegain,
    Midrange,
    Mill,
    Pillowfort,
    Ramp,
    Reanimator,
    Sacrifice,
    Spellslinger,
    Stax,
    Storm,
    Superfriends,
    Tempo,
    Tokens,
    Toolbox,
    Treasure,
    Tribal,
    Voltron,
}

impl DeckTag {
    /// All deck tag variants, alphabetical.
    pub fn all() -> &'static [DeckTag] {
        &[
            Self::Aggro,
            Self::Aristocrats,
            Self::Artifacts,
            Self::Blink,
            Self::Burn,
            Self::Combo,
            Self::Control,
            Self::Counters,
            Self::Enchantress,
            Self::Energy,
            Self::Graveyard,
            Self::GroupHug,
            Self::Infect,
            Self::Landfall,
            Self::Lifedrain,
            Self::Lifegain,
            Self::Midrange,
            Self::Mill,
            Self::Pillowfort,
            Self::Ramp,
            Self::Reanimator,
            Self::Sacrifice,
            Self::Spellslinger,
            Self::Stax,
            Self::Storm,
            Self::Superfriends,
            Self::Tempo,
            Self::Tokens,
            Self::Toolbox,
            Self::Treasure,
            Self::Tribal,
            Self::Voltron,
        ]
    }

    /// Human-readable display name.
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Aggro => "Aggro",
            Self::Aristocrats => "Aristocrats",
            Self::Artifacts => "Artifacts",
            Self::Blink => "Blink",
            Self::Burn => "Burn",
            Self::Combo => "Combo",
            Self::Control => "Control",
            Self::Counters => "+1/+1 Counters",
            Self::Enchantress => "Enchantress",
            Self::Energy => "Energy",
            Self::Graveyard => "Graveyard",
            Self::GroupHug => "Group Hug",
            Self::Infect => "Infect",
            Self::Landfall => "Landfall",
            Self::Lifedrain => "Lifedrain",
            Self::Lifegain => "Lifegain",
            Self::Midrange => "Midrange",
            Self::Mill => "Mill",
            Self::Pillowfort => "Pillowfort",
            Self::Ramp => "Ramp",
            Self::Reanimator => "Reanimator",
            Self::Sacrifice => "Sacrifice",
            Self::Spellslinger => "Spellslinger",
            Self::Stax => "Stax",
            Self::Storm => "Storm",
            Self::Superfriends => "Superfriends",
            Self::Tempo => "Tempo",
            Self::Tokens => "Tokens",
            Self::Toolbox => "Toolbox",
            Self::Treasure => "Treasure",
            Self::Tribal => "Tribal",
            Self::Voltron => "Voltron",
        }
    }
}

/// Display as snake_case (matches serde serialization).
impl fmt::Display for DeckTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = serde_json::to_string(self).unwrap_or_default();
        // serde_json wraps in quotes, strip them.
        write!(f, "{}", s.trim_matches('"'))
    }
}

/// Error when parsing an unrecognized deck tag string.
#[derive(Debug, thiserror::Error)]
#[error("unknown deck tag: {0}")]
pub struct InvalidDeckTag(pub String);

impl TryFrom<&str> for DeckTag {
    type Error = InvalidDeckTag;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        serde_json::from_str(&format!("\"{value}\"")).map_err(|_| InvalidDeckTag(value.to_string()))
    }
}

/// Parses raw tag strings into validated tags, dropping duplicates while
/// preserving order. Returns an error on the first unrecognized string. Callers
/// enforce [`MAX_DECK_TAGS`] separately so they can surface their own error.
pub fn parse_tags(raw: &[String]) -> Result<Vec<DeckTag>, InvalidDeckTag> {
    let mut out: Vec<DeckTag> = Vec::with_capacity(raw.len());
    for s in raw {
        let tag = DeckTag::try_from(s.as_str())?;
        if !out.contains(&tag) {
            out.push(tag);
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serde_round_trip() {
        for tag in DeckTag::all() {
            let json = serde_json::to_string(tag).unwrap();
            let parsed: DeckTag = serde_json::from_str(&json).unwrap();
            assert_eq!(*tag, parsed);
        }
    }

    #[test]
    fn display_matches_serde() {
        assert_eq!(DeckTag::GroupHug.to_string(), "group_hug");
        assert_eq!(DeckTag::Aggro.to_string(), "aggro");
        assert_eq!(DeckTag::Superfriends.to_string(), "superfriends");
    }

    #[test]
    fn try_from_valid() {
        assert_eq!(DeckTag::try_from("group_hug").unwrap(), DeckTag::GroupHug);
        assert_eq!(DeckTag::try_from("aggro").unwrap(), DeckTag::Aggro);
    }

    #[test]
    fn try_from_invalid() {
        assert!(DeckTag::try_from("not_a_tag").is_err());
    }

    #[test]
    fn display_name_non_empty() {
        for tag in DeckTag::all() {
            assert!(!tag.display_name().is_empty());
        }
    }

    #[test]
    fn all_has_32_variants() {
        assert_eq!(DeckTag::all().len(), 32);
    }
}
