use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Error)]
#[error("invalid language")]
pub struct InvalidLanguage;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Language {
    English,
    Spanish,
    French,
    German,
    Italian,
    Portuguese,
    Japanese,
    Korean,
    Russian,
    SimplifiedChinese,
    TraditionalChinese,
}

impl Language {
    /// Returns the Scryfall API language code
    pub fn to_code(&self) -> &'static str {
        match self {
            Language::English => "en",
            Language::Spanish => "es",
            Language::French => "fr",
            Language::German => "de",
            Language::Italian => "it",
            Language::Portuguese => "pt",
            Language::Japanese => "ja",
            Language::Korean => "ko",
            Language::Russian => "ru",
            Language::SimplifiedChinese => "zhs",
            Language::TraditionalChinese => "zht",
        }
    }

    pub fn to_name(&self) -> &'static str {
        match self {
            Language::English => "English",
            Language::Spanish => "Spanish",
            Language::French => "French",
            Language::German => "German",
            Language::Italian => "Italian",
            Language::Portuguese => "Portuguese",
            Language::Japanese => "Japanese",
            Language::Korean => "Korean",
            Language::Russian => "Russian",
            Language::SimplifiedChinese => "Simplified Chinese",
            Language::TraditionalChinese => "Traditional Chinese",
        }
    }

    pub fn all() -> Vec<Self> {
        vec![
            Language::English,
            Language::Spanish,
            Language::French,
            Language::German,
            Language::Italian,
            Language::Portuguese,
            Language::Japanese,
            Language::Korean,
            Language::Russian,
            Language::SimplifiedChinese,
            Language::TraditionalChinese,
        ]
    }
}

impl TryFrom<&str> for Language {
    type Error = InvalidLanguage;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "en" | "english" => Ok(Language::English),
            "es" | "spanish" => Ok(Language::Spanish),
            "fr" | "french" => Ok(Language::French),
            "de" | "german" => Ok(Language::German),
            "it" | "italian" => Ok(Language::Italian),
            "pt" | "portuguese" => Ok(Language::Portuguese),
            "ja" | "japanese" => Ok(Language::Japanese),
            "ko" | "korean" => Ok(Language::Korean),
            "ru" | "russian" => Ok(Language::Russian),
            "zhs" | "simplified chinese" => Ok(Language::SimplifiedChinese),
            "zht" | "traditional chinese" => Ok(Language::TraditionalChinese),
            _ => Err(InvalidLanguage),
        }
    }
}

impl TryFrom<String> for Language {
    type Error = InvalidLanguage;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_ref())
    }
}

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_name())
    }
}

impl Serialize for Language {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.to_code().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Language {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Self::try_from(String::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}
