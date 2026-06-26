//! Deck archetype/strategy tags.
//!
//! A deck can be labeled with a small set of these to describe its overall
//! strategy (Aggro, Tokens, Reanimator, …). Distinct from the card-level
//! `MechanicalCategory`: these describe a whole deck's game plan, not a single
//! card's role. The set is curated (from EDHREC themes and Archidekt/Moxfield
//! tags) and fixed so tags stay clean and filterable.
//!
//! The list is large on purpose — the picker is a searchable typeahead, so more
//! options cost nothing on screen. Variants are only ever added, never removed
//! or renamed, so previously-stored tags keep parsing.

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
    AttackTriggers,
    Auras,
    Blink,
    Bounce,
    Burn,
    Cascade,
    Clone,
    Combo,
    Control,
    Counters,
    Counterspells,
    Cycling,
    Defenders,
    Devotion,
    Discard,
    Draw,
    Enchantress,
    Energy,
    Equipment,
    Etb,
    Flying,
    GoWide,
    Graveyard,
    GroupHug,
    GroupSlug,
    Hatebears,
    Infect,
    LandDestruction,
    Landfall,
    Lands,
    Lifedrain,
    Lifegain,
    Midrange,
    Mill,
    Monarch,
    Pillowfort,
    Ping,
    Poison,
    Politics,
    Prison,
    Proliferate,
    Ramp,
    Reanimator,
    Removal,
    Sacrifice,
    Sagas,
    SelfMill,
    Spellslinger,
    Stax,
    Storm,
    Superfriends,
    Tempo,
    Theft,
    Tokens,
    Toolbox,
    Treasure,
    Tribal,
    Untap,
    Vehicles,
    Voltron,
    Wheels,
    Wipe,
}

impl DeckTag {
    /// All deck tag variants, alphabetical.
    pub fn all() -> &'static [DeckTag] {
        &[
            Self::Aggro,
            Self::Aristocrats,
            Self::Artifacts,
            Self::AttackTriggers,
            Self::Auras,
            Self::Blink,
            Self::Bounce,
            Self::Burn,
            Self::Cascade,
            Self::Clone,
            Self::Combo,
            Self::Control,
            Self::Counters,
            Self::Counterspells,
            Self::Cycling,
            Self::Defenders,
            Self::Devotion,
            Self::Discard,
            Self::Draw,
            Self::Enchantress,
            Self::Energy,
            Self::Equipment,
            Self::Etb,
            Self::Flying,
            Self::GoWide,
            Self::Graveyard,
            Self::GroupHug,
            Self::GroupSlug,
            Self::Hatebears,
            Self::Infect,
            Self::LandDestruction,
            Self::Landfall,
            Self::Lands,
            Self::Lifedrain,
            Self::Lifegain,
            Self::Midrange,
            Self::Mill,
            Self::Monarch,
            Self::Pillowfort,
            Self::Ping,
            Self::Poison,
            Self::Politics,
            Self::Prison,
            Self::Proliferate,
            Self::Ramp,
            Self::Reanimator,
            Self::Removal,
            Self::Sacrifice,
            Self::Sagas,
            Self::SelfMill,
            Self::Spellslinger,
            Self::Stax,
            Self::Storm,
            Self::Superfriends,
            Self::Tempo,
            Self::Theft,
            Self::Tokens,
            Self::Toolbox,
            Self::Treasure,
            Self::Tribal,
            Self::Untap,
            Self::Vehicles,
            Self::Voltron,
            Self::Wheels,
            Self::Wipe,
        ]
    }

    /// Human-readable display name.
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Aggro => "Aggro",
            Self::Aristocrats => "Aristocrats",
            Self::Artifacts => "Artifacts",
            Self::AttackTriggers => "Attack Triggers",
            Self::Auras => "Auras",
            Self::Blink => "Blink",
            Self::Bounce => "Bounce",
            Self::Burn => "Burn",
            Self::Cascade => "Cascade",
            Self::Clone => "Clone",
            Self::Combo => "Combo",
            Self::Control => "Control",
            Self::Counters => "+1/+1 Counters",
            Self::Counterspells => "Counterspells",
            Self::Cycling => "Cycling",
            Self::Defenders => "Defenders",
            Self::Devotion => "Devotion",
            Self::Discard => "Discard",
            Self::Draw => "Draw",
            Self::Enchantress => "Enchantress",
            Self::Energy => "Energy",
            Self::Equipment => "Equipment",
            Self::Etb => "ETB",
            Self::Flying => "Flying",
            Self::GoWide => "Go Wide",
            Self::Graveyard => "Graveyard",
            Self::GroupHug => "Group Hug",
            Self::GroupSlug => "Group Slug",
            Self::Hatebears => "Hatebears",
            Self::Infect => "Infect",
            Self::LandDestruction => "Land Destruction",
            Self::Landfall => "Landfall",
            Self::Lands => "Lands Matter",
            Self::Lifedrain => "Lifedrain",
            Self::Lifegain => "Lifegain",
            Self::Midrange => "Midrange",
            Self::Mill => "Mill",
            Self::Monarch => "Monarch",
            Self::Pillowfort => "Pillowfort",
            Self::Ping => "Pingers",
            Self::Poison => "Poison",
            Self::Politics => "Politics",
            Self::Prison => "Prison",
            Self::Proliferate => "Proliferate",
            Self::Ramp => "Ramp",
            Self::Reanimator => "Reanimator",
            Self::Removal => "Removal",
            Self::Sacrifice => "Sacrifice",
            Self::Sagas => "Sagas",
            Self::SelfMill => "Self-Mill",
            Self::Spellslinger => "Spellslinger",
            Self::Stax => "Stax",
            Self::Storm => "Storm",
            Self::Superfriends => "Superfriends",
            Self::Tempo => "Tempo",
            Self::Theft => "Theft",
            Self::Tokens => "Tokens",
            Self::Toolbox => "Toolbox",
            Self::Treasure => "Treasure",
            Self::Tribal => "Tribal",
            Self::Untap => "Untap",
            Self::Vehicles => "Vehicles",
            Self::Voltron => "Voltron",
            Self::Wheels => "Wheels",
            Self::Wipe => "Board Wipes",
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
        assert_eq!(DeckTag::AttackTriggers.to_string(), "attack_triggers");
        assert_eq!(DeckTag::Etb.to_string(), "etb");
    }

    #[test]
    fn try_from_valid() {
        assert_eq!(DeckTag::try_from("group_hug").unwrap(), DeckTag::GroupHug);
        assert_eq!(DeckTag::try_from("clone").unwrap(), DeckTag::Clone);
        assert_eq!(DeckTag::try_from("draw").unwrap(), DeckTag::Draw);
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
    fn all_variants_have_unique_display_names() {
        let names: std::collections::HashSet<_> =
            DeckTag::all().iter().map(|t| t.display_name()).collect();
        assert_eq!(names.len(), DeckTag::all().len());
    }
}
