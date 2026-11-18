//! A card's language as a collection row holds it, in Scryfall's codes.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

/// A printing's language, stored as Scryfall's code (`en`, `ja`, `zhs`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CardLanguage {
    /// English.
    En,
    /// Spanish.
    Es,
    /// French.
    Fr,
    /// German.
    De,
    /// Italian.
    It,
    /// Portuguese.
    Pt,
    /// Japanese.
    Ja,
    /// Korean.
    Ko,
    /// Russian.
    Ru,
    /// Simplified Chinese.
    Zhs,
    /// Traditional Chinese.
    Zht,
    /// Hebrew.
    He,
    /// Latin.
    La,
    /// Ancient Greek.
    Grc,
    /// Arabic.
    Ar,
    /// Sanskrit.
    Sa,
    /// Phyrexian.
    Ph,
    /// Quenya.
    Qya,
    /// A value this build doesn't know, from a newer server. The server never stores or sends
    /// it for a value it knows.
    #[serde(other)]
    Unknown,
}

impl CardLanguage {
    /// Every language Scryfall prints.
    pub const ALL: [Self; 18] = [
        Self::En,
        Self::Es,
        Self::Fr,
        Self::De,
        Self::It,
        Self::Pt,
        Self::Ja,
        Self::Ko,
        Self::Ru,
        Self::Zhs,
        Self::Zht,
        Self::He,
        Self::La,
        Self::Grc,
        Self::Ar,
        Self::Sa,
        Self::Ph,
        Self::Qya,
    ];

    /// Scryfall's code, the stored spelling.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::En => "en",
            Self::Es => "es",
            Self::Fr => "fr",
            Self::De => "de",
            Self::It => "it",
            Self::Pt => "pt",
            Self::Ja => "ja",
            Self::Ko => "ko",
            Self::Ru => "ru",
            Self::Zhs => "zhs",
            Self::Zht => "zht",
            Self::He => "he",
            Self::La => "la",
            Self::Grc => "grc",
            Self::Ar => "ar",
            Self::Sa => "sa",
            Self::Ph => "ph",
            Self::Qya => "qya",
            Self::Unknown => "unknown",
        }
    }
}

impl fmt::Display for CardLanguage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A language no export spells that way.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("unknown language")]
pub struct UnknownLanguage;

impl FromStr for CardLanguage {
    type Err = UnknownLanguage;

    /// Reads an export's cell, ignoring case and surrounding spaces: Scryfall's codes (ManaBox
    /// writes `en`, Archidekt `EN`), the older `jp`, `kr`, `cs` and `ct`, and English names.
    fn from_str(cell: &str) -> Result<Self, Self::Err> {
        let key = cell.trim().to_ascii_lowercase();
        let alias = match key.as_str() {
            "english" => "en",
            "spanish" => "es",
            "french" => "fr",
            "german" => "de",
            "italian" => "it",
            "portuguese" => "pt",
            "jp" | "japanese" => "ja",
            "kr" | "korean" => "ko",
            "russian" => "ru",
            "cs" | "zh" | "zh-cn" | "zh_cn" | "chinese simplified" | "simplified chinese" => "zhs",
            "ct" | "zh-tw" | "zh_tw" | "chinese traditional" | "traditional chinese" => "zht",
            other => other,
        };
        Self::ALL
            .into_iter()
            .find(|language| language.as_str() == alias)
            .ok_or(UnknownLanguage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn both_exports_spellings_read() {
        for (cell, want) in [
            ("en", CardLanguage::En),
            ("EN", CardLanguage::En),
            ("jp", CardLanguage::Ja),
            ("JA", CardLanguage::Ja),
            ("CS", CardLanguage::Zhs),
            ("zht", CardLanguage::Zht),
            ("PH", CardLanguage::Ph),
            ("German", CardLanguage::De),
        ] {
            assert_eq!(cell.parse::<CardLanguage>(), Ok(want), "{cell:?}");
        }
        assert_eq!("".parse::<CardLanguage>(), Err(UnknownLanguage));
        assert_eq!("klingon".parse::<CardLanguage>(), Err(UnknownLanguage));
    }

    #[test]
    fn every_language_reads_back_from_its_code() {
        for language in CardLanguage::ALL {
            assert_eq!(language.as_str().parse::<CardLanguage>(), Ok(language));
            let wire = serde_json::to_string(&language).unwrap();
            assert_eq!(wire, format!("\"{language}\""));
        }
    }
}
