//! A card's condition as a collection row holds it: ManaBox's grades and Archidekt's, side by
//! side, so neither scale is folded into the other.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

/// A collection row's condition. ManaBox grades on Cardmarket's scale (mint down to poor) and
/// Archidekt on TCGplayer's (near mint down to damaged); each keeps its own grade.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CardCondition {
    /// Mint.
    Mint,
    /// Near mint.
    NearMint,
    /// Excellent (Cardmarket).
    Excellent,
    /// Good (Cardmarket).
    Good,
    /// Lightly played.
    LightlyPlayed,
    /// Moderately played (TCGplayer).
    ModeratelyPlayed,
    /// Played (Cardmarket).
    Played,
    /// Heavily played (TCGplayer).
    HeavilyPlayed,
    /// Poor (Cardmarket).
    Poor,
    /// Damaged (TCGplayer).
    Damaged,
    /// A value this build doesn't know, from a newer server. The server never stores or sends
    /// it for a value it knows.
    #[serde(other)]
    Unknown,
}

impl CardCondition {
    /// Every grade.
    pub const ALL: [Self; 10] = [
        Self::Mint,
        Self::NearMint,
        Self::Excellent,
        Self::Good,
        Self::LightlyPlayed,
        Self::ModeratelyPlayed,
        Self::Played,
        Self::HeavilyPlayed,
        Self::Poor,
        Self::Damaged,
    ];

    /// The stored spelling.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Mint => "mint",
            Self::NearMint => "near_mint",
            Self::Excellent => "excellent",
            Self::Good => "good",
            Self::LightlyPlayed => "lightly_played",
            Self::ModeratelyPlayed => "moderately_played",
            Self::Played => "played",
            Self::HeavilyPlayed => "heavily_played",
            Self::Poor => "poor",
            Self::Damaged => "damaged",
            Self::Unknown => "unknown",
        }
    }
}

impl fmt::Display for CardCondition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A condition no export spells that way.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("unknown condition")]
pub struct UnknownCondition;

impl FromStr for CardCondition {
    type Err = UnknownCondition;

    /// Reads an export's cell, ignoring case, spaces and underscores: ManaBox writes
    /// `near_mint` or `light_played`, Archidekt `NM`, `LP`, `MP`, `HP` or `D`.
    fn from_str(cell: &str) -> Result<Self, Self::Err> {
        let key: String = cell
            .chars()
            .filter(char::is_ascii_alphanumeric)
            .map(|c| c.to_ascii_lowercase())
            .collect();
        match key.as_str() {
            "m" | "mint" => Ok(Self::Mint),
            "nm" | "nearmint" => Ok(Self::NearMint),
            "ex" | "excellent" => Ok(Self::Excellent),
            "gd" | "good" => Ok(Self::Good),
            "lp" | "lightplayed" | "lightlyplayed" => Ok(Self::LightlyPlayed),
            "mp" | "moderatelyplayed" => Ok(Self::ModeratelyPlayed),
            "pl" | "played" => Ok(Self::Played),
            "hp" | "heavilyplayed" => Ok(Self::HeavilyPlayed),
            "po" | "poor" => Ok(Self::Poor),
            "d" | "dmg" | "damaged" => Ok(Self::Damaged),
            _ => Err(UnknownCondition),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn both_exports_spellings_read() {
        for (cell, want) in [
            ("near_mint", CardCondition::NearMint),
            ("NM", CardCondition::NearMint),
            ("light_played", CardCondition::LightlyPlayed),
            ("LP", CardCondition::LightlyPlayed),
            ("MP", CardCondition::ModeratelyPlayed),
            ("HP", CardCondition::HeavilyPlayed),
            ("D", CardCondition::Damaged),
            ("excellent", CardCondition::Excellent),
            ("poor", CardCondition::Poor),
        ] {
            assert_eq!(cell.parse::<CardCondition>(), Ok(want), "{cell:?}");
        }
        assert_eq!("".parse::<CardCondition>(), Err(UnknownCondition));
        assert_eq!("shiny".parse::<CardCondition>(), Err(UnknownCondition));
    }

    #[test]
    fn every_grade_reads_back_from_its_stored_spelling() {
        for grade in CardCondition::ALL {
            assert_eq!(grade.as_str().parse::<CardCondition>(), Ok(grade));
            let wire = serde_json::to_string(&grade).unwrap();
            assert_eq!(wire, format!("\"{grade}\""));
        }
    }
}
