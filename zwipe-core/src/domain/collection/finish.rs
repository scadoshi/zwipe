//! A printing's finish as a collection row holds it.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

/// A collection row's finish. Wire and stored spellings are `nonfoil`, `foil` and `etched`.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum Finish {
    /// No foil.
    #[default]
    Nonfoil,
    /// Traditional foil.
    Foil,
    /// Etched foil.
    Etched,
    /// A value this build doesn't know, from a newer server. The server never stores or sends
    /// it for a value it knows.
    #[serde(other)]
    Unknown,
}

impl Finish {
    /// The stored spelling.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Nonfoil => "nonfoil",
            Self::Foil => "foil",
            Self::Etched => "etched",
            Self::Unknown => "unknown",
        }
    }
}

impl fmt::Display for Finish {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A finish no export spells that way.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("unknown finish")]
pub struct UnknownFinish;

impl FromStr for Finish {
    type Err = UnknownFinish;

    /// Reads an export's cell, ignoring case and surrounding spaces: ManaBox writes `normal`,
    /// `foil` and `etched`, Archidekt `Normal`, `Foil` and `Etched`. An empty cell is no foil.
    fn from_str(cell: &str) -> Result<Self, Self::Err> {
        match cell.trim().to_ascii_lowercase().as_str() {
            "" | "normal" | "nonfoil" | "non-foil" => Ok(Self::Nonfoil),
            "foil" => Ok(Self::Foil),
            "etched" | "etched foil" => Ok(Self::Etched),
            _ => Err(UnknownFinish),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn both_exports_spellings_read() {
        for (cell, want) in [
            ("normal", Finish::Nonfoil),
            ("Normal", Finish::Nonfoil),
            ("", Finish::Nonfoil),
            ("foil", Finish::Foil),
            (" Foil ", Finish::Foil),
            ("Etched", Finish::Etched),
        ] {
            assert_eq!(cell.parse::<Finish>(), Ok(want), "{cell:?}");
        }
        assert_eq!("gilded".parse::<Finish>(), Err(UnknownFinish));
    }

    #[test]
    fn the_wire_spelling_round_trips() {
        for finish in [Finish::Nonfoil, Finish::Foil, Finish::Etched] {
            let wire = serde_json::to_string(&finish).unwrap();
            assert_eq!(wire, format!("\"{finish}\""));
            assert_eq!(serde_json::from_str::<Finish>(&wire).unwrap(), finish);
        }
    }
}
