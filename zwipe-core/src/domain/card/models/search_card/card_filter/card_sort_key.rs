use crate::domain::card::scryfall_data::ScryfallData;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CardSortKey {
    Name,
    Cmc,
    Power,
    Toughness,
    Rarity,
    ReleasedAt,
    PriceUsd,
    PriceEur,
    PriceTix,
    EdhrecRank,
    Random,
}

#[allow(missing_docs)]
impl CardSortKey {
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
            Self::EdhrecRank,
            Self::Random,
        ]
    }
}

impl CardSortKey {
    /// Ascending comparison of two cards under this key. Missing/non-numeric
    /// values sort last (ascending); `Random` compares equal — shuffling is the
    /// collection's job, not a pairwise ordering.
    ///
    /// Shared by [`Cards::sorted`](crate::domain::card::search_card::cards::Cards::sorted)
    /// and deck-entry sorting so every in-memory sort agrees.
    pub fn compare(self, a: &ScryfallData, b: &ScryfallData) -> Ordering {
        /// Parses an optional string stat, treating missing/non-numeric as MAX
        /// so those cards sort last in ascending order.
        fn stat_i32(v: Option<&str>) -> i32 {
            v.and_then(|s| s.parse::<i32>().ok()).unwrap_or(i32::MAX)
        }
        fn price_f64(v: Option<&str>) -> f64 {
            v.and_then(|s| s.parse::<f64>().ok()).unwrap_or(f64::MAX)
        }
        fn cmp_f64(a: f64, b: f64) -> Ordering {
            a.partial_cmp(&b).unwrap_or(Ordering::Equal)
        }

        match self {
            Self::Name => a.name.cmp(&b.name),
            Self::Cmc => cmp_f64(a.cmc.unwrap_or(f64::MAX), b.cmc.unwrap_or(f64::MAX)),
            Self::Power => stat_i32(a.power.as_deref()).cmp(&stat_i32(b.power.as_deref())),
            Self::Toughness => {
                stat_i32(a.toughness.as_deref()).cmp(&stat_i32(b.toughness.as_deref()))
            }
            Self::Rarity => a.rarity.cmp(&b.rarity),
            Self::ReleasedAt => a.released_at.cmp(&b.released_at),
            Self::PriceUsd => cmp_f64(
                price_f64(a.prices.usd.as_deref()),
                price_f64(b.prices.usd.as_deref()),
            ),
            Self::PriceEur => cmp_f64(
                price_f64(a.prices.eur.as_deref()),
                price_f64(b.prices.eur.as_deref()),
            ),
            Self::PriceTix => cmp_f64(
                price_f64(a.prices.tix.as_deref()),
                price_f64(b.prices.tix.as_deref()),
            ),
            // Lower rank = more played; unranked sorts last (ascending).
            Self::EdhrecRank => a
                .edhrec_rank
                .unwrap_or(i32::MAX)
                .cmp(&b.edhrec_rank.unwrap_or(i32::MAX)),
            Self::Random => Ordering::Equal,
        }
    }
}

impl std::fmt::Display for CardSortKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Name => write!(f, "Name"),
            Self::Cmc => write!(f, "Mana value"),
            Self::Power => write!(f, "Power"),
            Self::Toughness => write!(f, "Toughness"),
            Self::Rarity => write!(f, "Rarity"),
            Self::ReleasedAt => write!(f, "Release Date"),
            Self::PriceUsd => write!(f, "Price (USD)"),
            Self::PriceEur => write!(f, "Price (EUR)"),
            Self::PriceTix => write!(f, "Price (TIX)"),
            Self::EdhrecRank => write!(f, "Popularity (EDHREC)"),
            Self::Random => write!(f, "Random"),
        }
    }
}
