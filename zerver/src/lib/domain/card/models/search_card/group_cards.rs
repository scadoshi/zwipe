//! In-memory card grouping for local `Vec<Card>` slices.
//!
//! Partitions a `Vec<Card>` into labelled groups based on card type, mana value,
//! or color identity. Works alongside `filter_cards.rs` — the caller is expected
//! to `filter_by` first (which handles sorting), then `group_by`.
//!
//! # Example
//!
//! ```rust,ignore
//! use zwipe::domain::card::models::search_card::group_cards::{GroupCards, GroupByOption};
//!
//! let groups = deck_cards.group_by(GroupByOption::CardType);
//! for group in &groups {
//!     println!("{} · {}", group.label, group.cards.len());
//! }
//! ```

use crate::domain::card::models::{
    Card,
    scryfall_data::colors::Color,
};

/// Grouping strategies for partitioning cards.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GroupByOption {
    /// Group by card type (Land, Creature, Planeswalker, etc.).
    CardType,
    /// Group by converted mana cost (0–5, 6+).
    Cmc,
    /// Group by color identity (WUBRG, multicolor, colorless).
    Color,
}

impl GroupByOption {
    /// Returns all grouping options.
    pub fn all() -> Vec<Self> {
        vec![Self::CardType, Self::Cmc, Self::Color]
    }
}

impl std::fmt::Display for GroupByOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CardType => write!(f, "type"),
            Self::Cmc => write!(f, "cmc"),
            Self::Color => write!(f, "color"),
        }
    }
}

/// A labelled group of cards.
#[derive(Debug, Clone)]
pub struct CardGroup {
    /// Display label for this group (e.g., "creatures", "3", "blue").
    pub label: String,
    /// Cards belonging to this group, in the order they were received.
    pub cards: Vec<Card>,
}

/// Extension trait for grouping a `Vec<Card>` into labelled buckets.
pub trait GroupCards {
    /// Partitions cards into groups according to the given option.
    ///
    /// Groups are emitted in a fixed order (not alphabetical). Empty groups are
    /// skipped. Card order within each group is preserved from the input.
    fn group_by(self, option: GroupByOption) -> Vec<CardGroup>;
}

impl GroupCards for Vec<Card> {
    fn group_by(self, option: GroupByOption) -> Vec<CardGroup> {
        let labels: Vec<&str> = match option {
            GroupByOption::CardType => vec![
                "lands",
                "creatures",
                "planeswalkers",
                "artifacts",
                "enchantments",
                "instants",
                "sorceries",
                "other",
            ],
            GroupByOption::Cmc => vec!["0", "1", "2", "3", "4", "5", "6+"],
            GroupByOption::Color => vec![
                "white",
                "blue",
                "black",
                "red",
                "green",
                "multicolor",
                "colorless",
            ],
        };

        let mut buckets: Vec<Vec<Card>> = vec![Vec::new(); labels.len()];

        for card in self {
            let idx = classify(&card, option);
            buckets[idx].push(card);
        }

        labels
            .into_iter()
            .zip(buckets)
            .filter(|(_, cards)| !cards.is_empty())
            .map(|(label, cards)| CardGroup {
                label: label.to_string(),
                cards,
            })
            .collect()
    }
}

/// Returns the bucket index for a card under the given grouping option.
fn classify(card: &Card, option: GroupByOption) -> usize {
    match option {
        GroupByOption::CardType => classify_card_type(card),
        GroupByOption::Cmc => classify_cmc(card),
        GroupByOption::Color => classify_color(card),
    }
}

/// Card type classification — first match wins.
///
/// Priority: Land → Creature → Planeswalker → Artifact → Enchantment →
/// Instant → Sorcery → Other.
fn classify_card_type(card: &Card) -> usize {
    let type_line = match &card.scryfall_data.type_line {
        Some(tl) => tl.as_str(),
        None => return 7, // "other"
    };

    const CHECKS: &[&str] = &[
        "Land",
        "Creature",
        "Planeswalker",
        "Artifact",
        "Enchantment",
        "Instant",
        "Sorcery",
    ];

    CHECKS
        .iter()
        .position(|keyword| type_line.contains(keyword))
        .unwrap_or(7) // "other"
}

/// CMC classification — floor to integer, cap at 6.
fn classify_cmc(card: &Card) -> usize {
    let cmc = card.scryfall_data.cmc.unwrap_or(0.0);
    let floored = cmc.floor() as usize;
    floored.min(6) // indices 0–5 for "0"–"5", index 6 for "6+"
}

/// Color identity classification — uses WUBRG order + multicolor + colorless.
fn classify_color(card: &Card) -> usize {
    let ci = &card.scryfall_data.color_identity;

    if ci.is_empty() {
        return 6; // "colorless"
    }

    if ci.len() >= 2 {
        return 5; // "multicolor"
    }

    // Exactly one color — map to WUBRG index (0–4)
    match ci[0] {
        Color::White => 0,
        Color::Blue => 1,
        Color::Black => 2,
        Color::Red => 3,
        Color::Green => 4,
    }
}
