//! Secondary, non-gameplay deck labels.
//!
//! Distinct from [`DeckTag`](super::deck_tag::DeckTag) (what the deck *does*,
//! the gameplay axis) and [`PowerLevel`](super::power_level::PowerLevel) (a
//! single-select rating). This is the multi-select bucket for descriptors that
//! aren't about the game plan — Budget, Jank, Meme, Precon, … — and is meant to
//! grow freely over time.
//!
//! Variants are only ever added, never removed or renamed, so previously-stored
//! values keep parsing.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Maximum number of other-tags a single deck may carry.
pub const MAX_DECK_OTHER_TAGS: usize = 5;

/// A secondary, non-gameplay deck label.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeckOtherTag {
    Budget,
    Jank,
    Meme,
    Precon,
    UpgradedPrecon,
}

impl DeckOtherTag {
    /// All other-tag variants, alphabetical.
    pub fn all() -> &'static [DeckOtherTag] {
        &[
            Self::Budget,
            Self::Jank,
            Self::Meme,
            Self::Precon,
            Self::UpgradedPrecon,
        ]
    }

    /// Human-readable display name.
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Budget => "Budget",
            Self::Jank => "Jank",
            Self::Meme => "Meme",
            Self::Precon => "Precon",
            Self::UpgradedPrecon => "Upgraded Precon",
        }
    }

    /// One-line, plain-language description, for the picker's hint dialog.
    pub fn description(&self) -> &'static str {
        match self {
            Self::Budget => "Built to a tight price — cheap cards over expensive staples",
            Self::Jank => "Embraces janky, suboptimal cards for the fun of it",
            Self::Meme => "A joke deck built around a bit, not around winning",
            Self::Precon => "An unmodified preconstructed deck, played as it came",
            Self::UpgradedPrecon => "A preconstructed deck with a round of upgrades",
        }
    }
}

/// Display as snake_case (matches serde serialization).
impl fmt::Display for DeckOtherTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = serde_json::to_string(self).unwrap_or_default();
        write!(f, "{}", s.trim_matches('"'))
    }
}

/// Error when parsing an unrecognized other-tag string.
#[derive(Debug, thiserror::Error)]
#[error("unknown deck other-tag: {0}")]
pub struct InvalidDeckOtherTag(pub String);

impl TryFrom<&str> for DeckOtherTag {
    type Error = InvalidDeckOtherTag;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        serde_json::from_str(&format!("\"{value}\""))
            .map_err(|_| InvalidDeckOtherTag(value.to_string()))
    }
}

/// Parses raw strings into validated other-tags, dropping duplicates while
/// preserving order. Returns an error on the first unrecognized string. Callers
/// enforce [`MAX_DECK_OTHER_TAGS`] separately so they can surface their own error.
pub fn parse_other_tags(raw: &[String]) -> Result<Vec<DeckOtherTag>, InvalidDeckOtherTag> {
    let mut out: Vec<DeckOtherTag> = Vec::with_capacity(raw.len());
    for s in raw {
        let tag = DeckOtherTag::try_from(s.as_str())?;
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
        for tag in DeckOtherTag::all() {
            let json = serde_json::to_string(tag).unwrap();
            let parsed: DeckOtherTag = serde_json::from_str(&json).unwrap();
            assert_eq!(*tag, parsed);
        }
    }

    #[test]
    fn display_matches_serde() {
        assert_eq!(DeckOtherTag::Budget.to_string(), "budget");
        assert_eq!(DeckOtherTag::UpgradedPrecon.to_string(), "upgraded_precon");
    }

    #[test]
    fn parses_and_dedupes() {
        let tags = parse_other_tags(&["budget".into(), "jank".into(), "budget".into()]).unwrap();
        assert_eq!(tags, vec![DeckOtherTag::Budget, DeckOtherTag::Jank]);
    }

    #[test]
    fn rejects_unknown() {
        assert!(parse_other_tags(&["not_a_tag".into()]).is_err());
    }

    #[test]
    fn display_names_unique() {
        let names: std::collections::HashSet<_> =
            DeckOtherTag::all().iter().map(|t| t.display_name()).collect();
        assert_eq!(names.len(), DeckOtherTag::all().len());
    }
}
