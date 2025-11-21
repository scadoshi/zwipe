use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CardType {
    Instant,
    Sorcery,
    Enchantment,
    Creature,
    Artifact,
    Planeswalker,
    Land,
}

impl Display for CardType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CardType::Instant => write!(f, "instant"),
            CardType::Sorcery => write!(f, "sorcery"),
            CardType::Enchantment => write!(f, "enchantment"),
            CardType::Creature => write!(f, "creature"),
            CardType::Artifact => write!(f, "artifact"),
            CardType::Planeswalker => write!(f, "planeswalker"),
            CardType::Land => write!(f, "land"),
        }
    }
}

pub trait WithCardTypes {
    fn with_all_card_types() -> Self;
}

impl WithCardTypes for Vec<CardType> {
    fn with_all_card_types() -> Self {
        vec![
            CardType::Artifact,
            CardType::Creature,
            CardType::Planeswalker,
            CardType::Instant,
            CardType::Sorcery,
            CardType::Enchantment,
            CardType::Land,
        ]
    }
}
