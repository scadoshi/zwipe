use serde::{Deserialize, Serialize};

#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderByOption {
    Name,
    Cmc,
    Power,
    Toughness,
    Rarity,
    ReleasedAt,
    PriceUsd,
    PriceEur,
    PriceTix,
    Random,
}

#[allow(missing_docs)]
impl OrderByOption {
    pub fn all() -> Vec<Self> {
        vec![
            Self::Name,
            Self::Cmc,
            Self::Power,
            Self::Toughness,
            Self::Rarity,
            Self::ReleasedAt,
            Self::PriceUsd,
            Self::PriceEur,
            Self::PriceTix,
            Self::Random,
        ]
    }
}

impl std::fmt::Display for OrderByOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Name => write!(f, "Name"),
            Self::Cmc => write!(f, "Mana Value"),
            Self::Power => write!(f, "Power"),
            Self::Toughness => write!(f, "Toughness"),
            Self::Rarity => write!(f, "Rarity"),
            Self::ReleasedAt => write!(f, "Release Date"),
            Self::PriceUsd => write!(f, "Price (USD)"),
            Self::PriceEur => write!(f, "Price (EUR)"),
            Self::PriceTix => write!(f, "Price (TIX)"),
            Self::Random => write!(f, "Random"),
        }
    }
}
