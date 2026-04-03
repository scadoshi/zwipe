use serde::{Deserialize, Serialize};
use std::fmt::Display;

/// Magic: The Gathering's main card types.
///
/// These are the fundamental card types that appear on the type line.
/// Cards can have multiple types (e.g., "Artifact Creature").
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum CardType {
    /// Instant spell (cast during any phase, resolves immediately).
    Instant,
    /// Sorcery spell (cast during main phase, resolves immediately).
    Sorcery,
    /// Enchantment permanent (persistent effect).
    Enchantment,
    /// Creature permanent (can attack/block, has power/toughness).
    Creature,
    /// Artifact permanent (usually colorless, represents magical items).
    Artifact,
    /// Planeswalker permanent (loyalty abilities, can be attacked).
    Planeswalker,
    /// Land permanent (produces mana, doesn't use the stack).
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

/// Extension trait to create collections containing all card types.
pub trait WithCardTypes {
    /// Creates a collection with all 7 main card types.
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
