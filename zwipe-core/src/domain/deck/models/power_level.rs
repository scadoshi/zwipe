//! Deck power level — the official WotC Commander Bracket a deck is built for.
//!
//! A single-select rating (not a tag): a deck has exactly one, and it's worth
//! sorting/filtering on. Distinct from [`DeckTag`](super::deck_tag::DeckTag)
//! (what the deck does) and [`DeckOtherTag`](super::deck_other_tag::DeckOtherTag)
//! (non-gameplay labels like Budget/Jank).
//!
//! Brackets are Commander-coined but players apply them loosely to any deck, so
//! this isn't gated by format. Variants are only ever added, never removed or
//! renamed, so previously-stored values keep parsing.

use serde::{Deserialize, Serialize};
use std::fmt;

/// The WotC Commander Bracket a deck targets (1–5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PowerLevel {
    /// Bracket 1.
    Exhibition,
    /// Bracket 2.
    Core,
    /// Bracket 3.
    Upgraded,
    /// Bracket 4.
    Optimized,
    /// Bracket 5.
    Cedh,
}

impl PowerLevel {
    /// All power levels, low to high (bracket 1 → 5).
    pub fn all() -> &'static [PowerLevel] {
        &[
            Self::Exhibition,
            Self::Core,
            Self::Upgraded,
            Self::Optimized,
            Self::Cedh,
        ]
    }

    /// The bracket number (1–5).
    pub fn bracket(&self) -> u8 {
        match self {
            Self::Exhibition => 1,
            Self::Core => 2,
            Self::Upgraded => 3,
            Self::Optimized => 4,
            Self::Cedh => 5,
        }
    }

    /// Human-readable display name, including the bracket number.
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Exhibition => "Exhibition (1)",
            Self::Core => "Core (2)",
            Self::Upgraded => "Upgraded (3)",
            Self::Optimized => "Optimized (4)",
            Self::Cedh => "cEDH (5)",
        }
    }

    /// One-line, plain-language description of the bracket, for the picker.
    pub fn description(&self) -> &'static str {
        match self {
            Self::Exhibition => "Ultra-casual showcase and theme decks; winning isn't the point",
            Self::Core => "Precon-level power — the average kitchen-table deck",
            Self::Upgraded => "Beyond precon: stronger cards and synergy, but not cutthroat",
            Self::Optimized => "High-power, no holds barred short of full competitive",
            Self::Cedh => "Competitive EDH — the strongest tuned meta decks",
        }
    }
}

/// Display as snake_case (matches serde serialization).
impl fmt::Display for PowerLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = serde_json::to_string(self).unwrap_or_default();
        write!(f, "{}", s.trim_matches('"'))
    }
}

/// Error when parsing an unrecognized power level string.
#[derive(Debug, thiserror::Error)]
#[error("unknown power level: {0}")]
pub struct InvalidPowerLevel(pub String);

impl TryFrom<&str> for PowerLevel {
    type Error = InvalidPowerLevel;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        serde_json::from_str(&format!("\"{value}\""))
            .map_err(|_| InvalidPowerLevel(value.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serde_round_trip() {
        for pl in PowerLevel::all() {
            let json = serde_json::to_string(pl).unwrap();
            let parsed: PowerLevel = serde_json::from_str(&json).unwrap();
            assert_eq!(*pl, parsed);
        }
    }

    #[test]
    fn display_matches_serde() {
        assert_eq!(PowerLevel::Exhibition.to_string(), "exhibition");
        assert_eq!(PowerLevel::Cedh.to_string(), "cedh");
    }

    #[test]
    fn try_from_valid_and_invalid() {
        assert_eq!(PowerLevel::try_from("optimized").unwrap(), PowerLevel::Optimized);
        assert!(PowerLevel::try_from("not_a_bracket").is_err());
    }

    #[test]
    fn brackets_are_one_through_five() {
        let nums: Vec<u8> = PowerLevel::all().iter().map(|p| p.bracket()).collect();
        assert_eq!(nums, vec![1, 2, 3, 4, 5]);
    }
}
