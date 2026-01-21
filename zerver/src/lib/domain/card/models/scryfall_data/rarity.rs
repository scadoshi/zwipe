use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Error)]
#[error("invalid rarity")]
pub struct InvalidRarity;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Rarity {
    Common,
    Uncommon,
    Rare,
    Mythic,
    Bonus,
    Special,
}

impl TryFrom<&str> for Rarity {
    type Error = InvalidRarity;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "common" | "c" => Ok(Self::Common),
            "uncommon" | "u" => Ok(Self::Uncommon),
            "rare" | "r" => Ok(Self::Rare),
            "mythic" | "mythic-rare" | "mythic_rare" | "mythic rare" | "m" => Ok(Self::Mythic),
            "bonus" | "b" => Ok(Self::Bonus),
            "special" | "s" => Ok(Self::Special),
            _ => Err(InvalidRarity),
        }
    }
}

impl TryFrom<String> for Rarity {
    type Error = InvalidRarity;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_ref())
    }
}

impl Rarity {
    pub fn to_long_name(&self) -> String {
        match self {
            Self::Common => "Common".to_string(),
            Self::Uncommon => "Uncommon".to_string(),
            Self::Rare => "Rare".to_string(),
            Self::Mythic => "Mythic".to_string(),
            Self::Bonus => "Bonus".to_string(),
            Self::Special => "Special".to_string(),
        }
    }

    pub fn to_short_name(&self) -> String {
        match self {
            Self::Common => "C".to_string(),
            Self::Uncommon => "U".to_string(),
            Self::Rare => "R".to_string(),
            Self::Mythic => "M".to_string(),
            Self::Bonus => "B".to_string(),
            Self::Special => "S".to_string(),
        }
    }

    pub fn all() -> Vec<Self> {
        vec![
            Self::Common,
            Self::Uncommon,
            Self::Rare,
            Self::Mythic,
            Self::Bonus,
            Self::Special,
        ]
    }
}

impl std::fmt::Display for Rarity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_long_name())
    }
}

impl Serialize for Rarity {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.to_long_name().to_lowercase().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Rarity {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Self::try_from(String::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Rarities(Vec<Rarity>);

impl std::ops::Deref for Rarities {
    type Target = [Rarity];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Rarities {
    pub fn to_short_names(&self) -> Vec<String> {
        self.0.iter().map(|c| c.to_short_name()).collect()
    }

    pub fn to_long_names(&self) -> Vec<String> {
        self.0.iter().map(|c| c.to_long_name()).collect()
    }
}

impl FromIterator<Rarity> for Rarities {
    fn from_iter<T: IntoIterator<Item = Rarity>>(iter: T) -> Self {
        Rarities(iter.into_iter().collect())
    }
}

impl<I> From<I> for Rarities
where
    I: IntoIterator<Item = Rarity>,
{
    fn from(value: I) -> Self {
        value.into_iter().collect()
    }
}

impl Serialize for Rarities {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Rarities {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Vec::<Rarity>::deserialize(deserializer).map(Rarities)
    }
}
