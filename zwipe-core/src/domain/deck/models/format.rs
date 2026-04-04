//! Deck format classification for Magic: The Gathering.
//!
//! Each format defines deck-building rules: card pool legality, copy limits,
//! deck size constraints, and whether a commander is required.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Error returned when parsing an invalid format string.
#[derive(Debug, Clone, Error)]
#[error("invalid format")]
pub struct InvalidFormat;

/// Magic: The Gathering deck format.
///
/// Variants match the field names in [`Legalities`] for direct lookup.
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Standard,
    Future,
    Historic,
    Timeless,
    Gladiator,
    Pioneer,
    Modern,
    Legacy,
    Pauper,
    Vintage,
    Penny,
    Commander,
    Oathbreaker,
    StandardBrawl,
    Brawl,
    Alchemy,
    PauperCommander,
    Duel,
    OldSchool,
    Premodern,
    Predh,
    Explorer,
    HistoricBrawl,
}

impl Format {
    /// Returns the key used in the legalities JSONB (matches Legalities struct field names).
    pub fn to_legality_key(&self) -> &str {
        match self {
            Self::Standard => "standard",
            Self::Future => "future",
            Self::Historic => "historic",
            Self::Timeless => "timeless",
            Self::Gladiator => "gladiator",
            Self::Pioneer => "pioneer",
            Self::Modern => "modern",
            Self::Legacy => "legacy",
            Self::Pauper => "pauper",
            Self::Vintage => "vintage",
            Self::Penny => "penny",
            Self::Commander => "commander",
            Self::Oathbreaker => "oathbreaker",
            Self::StandardBrawl => "standardbrawl",
            Self::Brawl => "brawl",
            Self::Alchemy => "alchemy",
            Self::PauperCommander => "paupercommander",
            Self::Duel => "duel",
            Self::OldSchool => "oldschool",
            Self::Premodern => "premodern",
            Self::Predh => "predh",
            Self::Explorer => "explorer",
            Self::HistoricBrawl => "historicbrawl",
        }
    }

    /// Human-readable display name.
    pub fn display_name(&self) -> &str {
        match self {
            Self::Standard => "Standard",
            Self::Future => "Future",
            Self::Historic => "Historic",
            Self::Timeless => "Timeless",
            Self::Gladiator => "Gladiator",
            Self::Pioneer => "Pioneer",
            Self::Modern => "Modern",
            Self::Legacy => "Legacy",
            Self::Pauper => "Pauper",
            Self::Vintage => "Vintage",
            Self::Penny => "Penny Dreadful",
            Self::Commander => "Commander",
            Self::Oathbreaker => "Oathbreaker",
            Self::StandardBrawl => "Standard Brawl",
            Self::Brawl => "Brawl",
            Self::Alchemy => "Alchemy",
            Self::PauperCommander => "Pauper Commander",
            Self::Duel => "Duel Commander",
            Self::OldSchool => "Old School",
            Self::Premodern => "Premodern",
            Self::Predh => "PreDH",
            Self::Explorer => "Explorer",
            Self::HistoricBrawl => "Historic Brawl",
        }
    }

    /// Minimum number of cards required by this format.
    pub fn min_cards(&self) -> Option<u32> {
        match self {
            Self::Commander | Self::PauperCommander | Self::Duel | Self::Predh => Some(100),
            Self::Brawl | Self::StandardBrawl | Self::HistoricBrawl | Self::Oathbreaker => {
                Some(60)
            }
            Self::Standard | Self::Pioneer | Self::Modern | Self::Legacy | Self::Vintage
            | Self::Pauper | Self::OldSchool | Self::Premodern | Self::Explorer
            | Self::Alchemy | Self::Historic | Self::Timeless | Self::Future | Self::Penny
            | Self::Gladiator => Some(60),
        }
    }

    /// Maximum number of cards allowed by this format. `None` means no maximum.
    pub fn max_cards(&self) -> Option<u32> {
        match self {
            Self::Commander | Self::PauperCommander | Self::Duel | Self::Predh => Some(100),
            Self::Brawl | Self::StandardBrawl | Self::HistoricBrawl => Some(60),
            _ => None,
        }
    }

    /// Maximum copies of a single non-basic-land card.
    pub fn copy_max(&self) -> u32 {
        match self {
            Self::Commander | Self::Brawl | Self::StandardBrawl | Self::HistoricBrawl
            | Self::PauperCommander | Self::Duel | Self::Predh | Self::Oathbreaker
            | Self::Gladiator => 1,
            _ => 4,
        }
    }

    /// Whether this format requires a commander.
    pub fn has_commander(&self) -> bool {
        matches!(
            self,
            Self::Commander
                | Self::Brawl
                | Self::StandardBrawl
                | Self::HistoricBrawl
                | Self::PauperCommander
                | Self::Duel
                | Self::Predh
                | Self::Oathbreaker
        )
    }

    /// Whether this format enforces color identity based on the commander.
    pub fn checks_color_identity(&self) -> bool {
        self.has_commander()
    }

    /// Formats that require a commander, alphabetical.
    pub fn commander_formats() -> &'static [Format] {
        &[
            Self::Brawl,
            Self::Commander,
            Self::Duel,
            Self::HistoricBrawl,
            Self::Oathbreaker,
            Self::PauperCommander,
            Self::Predh,
            Self::StandardBrawl,
        ]
    }

    /// All format variants, commander formats first then alphabetical.
    pub fn all() -> &'static [Format] {
        &[
            // commander formats
            Self::Brawl,
            Self::Commander,
            Self::Duel,
            Self::HistoricBrawl,
            Self::Oathbreaker,
            Self::PauperCommander,
            Self::Predh,
            Self::StandardBrawl,
            // non-commander formats alphabetical
            Self::Alchemy,
            Self::Explorer,
            Self::Future,
            Self::Gladiator,
            Self::Historic,
            Self::Legacy,
            Self::Modern,
            Self::OldSchool,
            Self::Pauper,
            Self::Penny,
            Self::Pioneer,
            Self::Premodern,
            Self::Standard,
            Self::Timeless,
            Self::Vintage,
        ]
    }
}

impl TryFrom<&str> for Format {
    type Error = InvalidFormat;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "standard" => Ok(Self::Standard),
            "future" => Ok(Self::Future),
            "historic" => Ok(Self::Historic),
            "timeless" => Ok(Self::Timeless),
            "gladiator" => Ok(Self::Gladiator),
            "pioneer" => Ok(Self::Pioneer),
            "modern" => Ok(Self::Modern),
            "legacy" => Ok(Self::Legacy),
            "pauper" => Ok(Self::Pauper),
            "vintage" => Ok(Self::Vintage),
            "penny" => Ok(Self::Penny),
            "commander" => Ok(Self::Commander),
            "oathbreaker" => Ok(Self::Oathbreaker),
            "standardbrawl" | "standard_brawl" | "standard brawl" => Ok(Self::StandardBrawl),
            "brawl" => Ok(Self::Brawl),
            "alchemy" => Ok(Self::Alchemy),
            "paupercommander" | "pauper_commander" | "pauper commander" => {
                Ok(Self::PauperCommander)
            }
            "duel" => Ok(Self::Duel),
            "oldschool" | "old_school" | "old school" => Ok(Self::OldSchool),
            "premodern" => Ok(Self::Premodern),
            "predh" => Ok(Self::Predh),
            "explorer" => Ok(Self::Explorer),
            "historicbrawl" | "historic_brawl" | "historic brawl" => Ok(Self::HistoricBrawl),
            _ => Err(InvalidFormat),
        }
    }
}

impl TryFrom<String> for Format {
    type Error = InvalidFormat;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_ref())
    }
}

impl std::fmt::Display for Format {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

impl Serialize for Format {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.to_legality_key().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Format {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Self::try_from(String::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_formats_round_trip_through_legality_key() {
        for format in Format::all() {
            let key = format.to_legality_key();
            let parsed = Format::try_from(key).unwrap();
            assert_eq!(*format, parsed);
        }
    }

    #[test]
    fn all_formats_round_trip_through_serde() {
        for format in Format::all() {
            let json = serde_json::to_string(format).unwrap();
            let parsed: Format = serde_json::from_str(&json).unwrap();
            assert_eq!(*format, parsed);
        }
    }

    #[test]
    fn commander_rules() {
        assert_eq!(Format::Commander.min_cards(), Some(100));
        assert_eq!(Format::Commander.max_cards(), Some(100));
        assert_eq!(Format::Commander.copy_max(), 1);
        assert!(Format::Commander.has_commander());
        assert!(Format::Commander.checks_color_identity());
    }

    #[test]
    fn standard_rules() {
        assert_eq!(Format::Standard.min_cards(), Some(60));
        assert_eq!(Format::Standard.max_cards(), None);
        assert_eq!(Format::Standard.copy_max(), 4);
        assert!(!Format::Standard.has_commander());
        assert!(!Format::Standard.checks_color_identity());
    }

    #[test]
    fn invalid_format_rejected() {
        assert!(Format::try_from("notaformat").is_err());
    }
}
