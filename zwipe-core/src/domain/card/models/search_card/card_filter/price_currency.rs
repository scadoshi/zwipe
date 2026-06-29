//! Currency selector for the price-range filter.

use serde::{Deserialize, Serialize};

/// Currency a price-range filter compares against. Maps to the JSONB key on
/// `scryfall_data.prices`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum PriceCurrency {
    /// US Dollars (TCGplayer) — the default.
    #[default]
    Usd,
    /// Euros (Cardmarket).
    Eur,
    /// MTGO Event Tickets.
    Tix,
}

impl PriceCurrency {
    /// JSONB key on `scryfall_data.prices` for this currency.
    pub fn json_key(&self) -> &'static str {
        match self {
            Self::Usd => "usd",
            Self::Eur => "eur",
            Self::Tix => "tix",
        }
    }

    /// Short label for the currency chips.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Usd => "USD",
            Self::Eur => "EUR",
            Self::Tix => "TIX",
        }
    }

    /// All currencies in display order.
    pub fn all() -> &'static [PriceCurrency] {
        &[Self::Usd, Self::Eur, Self::Tix]
    }
}
