//! What a collection row's copies cost when bought, and in which currency.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

/// A price that isn't a plain amount of at most two decimals between 0 and
/// [`PurchasePrice::MAX_CENTS`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("not a price")]
pub struct InvalidPurchasePrice;

/// A purchase price in hundredths of the currency (cents), 0 to [`PurchasePrice::MAX_CENTS`],
/// which the database stores as `NUMERIC(12, 2)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "i64", into = "i64")]
pub struct PurchasePrice(i64);

impl PurchasePrice {
    /// The largest price `NUMERIC(12, 2)` holds, 9,999,999,999.99, in cents.
    pub const MAX_CENTS: i64 = 999_999_999_999;

    /// A price of `cents`.
    ///
    /// # Errors
    ///
    /// Returns [`InvalidPurchasePrice`] below 0 or above [`PurchasePrice::MAX_CENTS`].
    pub fn from_cents(cents: i64) -> Result<Self, InvalidPurchasePrice> {
        if (0..=Self::MAX_CENTS).contains(&cents) {
            Ok(Self(cents))
        } else {
            Err(InvalidPurchasePrice)
        }
    }

    /// The price in cents.
    #[must_use]
    pub fn cents(self) -> i64 {
        self.0
    }
}

impl TryFrom<i64> for PurchasePrice {
    type Error = InvalidPurchasePrice;

    fn try_from(cents: i64) -> Result<Self, Self::Error> {
        Self::from_cents(cents)
    }
}

impl From<PurchasePrice> for i64 {
    fn from(price: PurchasePrice) -> Self {
        price.0
    }
}

impl FromStr for PurchasePrice {
    type Err = InvalidPurchasePrice;

    /// Reads an export's cell: digits, then optionally a point or a comma and one or two more
    /// (`0.41`, `12`, `3,5`), after an optional currency symbol and before an optional space and
    /// three-letter code (`$4.99`, `0.50 USD`). Anything else, a sign or a third decimal
    /// included, is not a price.
    fn from_str(cell: &str) -> Result<Self, Self::Err> {
        let cell = cell.trim().trim_start_matches(['$', '€', '£', '¥']);
        let amount = match cell.rsplit_once(' ') {
            Some((amount, code))
                if code.len() == 3 && code.chars().all(|c| c.is_ascii_alphabetic()) =>
            {
                amount.trim_end()
            }
            _ => cell,
        };
        let (whole, fraction) = match amount.split_once(['.', ',']) {
            Some((whole, fraction)) => (whole, fraction),
            None => (amount, ""),
        };
        let digits = |part: &str| part.chars().all(|c| c.is_ascii_digit());
        if whole.is_empty()
            || whole.len() > 12
            || fraction.len() > 2
            || !digits(whole)
            || !digits(fraction)
        {
            return Err(InvalidPurchasePrice);
        }
        let whole: i64 = whole.parse().map_err(|_| InvalidPurchasePrice)?;
        let fraction: i64 = match fraction.len() {
            0 => 0,
            1 => fraction
                .parse::<i64>()
                .map_err(|_| InvalidPurchasePrice)?
                .saturating_mul(10),
            _ => fraction.parse().map_err(|_| InvalidPurchasePrice)?,
        };
        let cents = whole
            .checked_mul(100)
            .and_then(|c| c.checked_add(fraction))
            .ok_or(InvalidPurchasePrice)?;
        Self::from_cents(cents)
    }
}

/// A currency that isn't three ASCII letters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("not a currency code")]
pub struct InvalidPurchaseCurrency;

/// The currency a purchase price is in: an ISO 4217 code, three uppercase letters (`USD`).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct PurchaseCurrency(String);

impl PurchaseCurrency {
    /// The code.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl FromStr for PurchaseCurrency {
    type Err = InvalidPurchaseCurrency;

    /// Reads an export's cell, uppercasing it: `usd` and `USD` are both `USD`.
    fn from_str(cell: &str) -> Result<Self, Self::Err> {
        let code = cell.trim().to_ascii_uppercase();
        if code.len() == 3 && code.chars().all(|c| c.is_ascii_uppercase()) {
            Ok(Self(code))
        } else {
            Err(InvalidPurchaseCurrency)
        }
    }
}

impl TryFrom<String> for PurchaseCurrency {
    type Error = InvalidPurchaseCurrency;

    fn try_from(code: String) -> Result<Self, Self::Error> {
        code.parse()
    }
}

impl From<PurchaseCurrency> for String {
    fn from(currency: PurchaseCurrency) -> Self {
        currency.0
    }
}

impl fmt::Display for PurchaseCurrency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn export_prices_read_as_cents() {
        for (cell, cents) in [
            ("0.41", 41),
            ("1.16", 116),
            ("12", 1_200),
            ("3,5", 350),
            ("$4.99", 499),
            ("0.5 USD", 50),
        ] {
            assert_eq!(
                cell.parse::<PurchasePrice>().map(PurchasePrice::cents),
                Ok(cents),
                "{cell:?}"
            );
        }
    }

    #[test]
    fn what_isnt_a_price_is_turned_down() {
        for cell in [
            "",
            "-1",
            "1.234",
            "abc",
            "1e5",
            "=1+1",
            "1.2.3",
            "9999999999999",
        ] {
            assert_eq!(
                cell.parse::<PurchasePrice>(),
                Err(InvalidPurchasePrice),
                "{cell:?}"
            );
        }
    }

    #[test]
    fn a_currency_is_three_letters() {
        assert_eq!("usd".parse::<PurchaseCurrency>().unwrap().as_str(), "USD");
        for cell in ["", "US", "USDX", "U$D", "€"] {
            assert!(cell.parse::<PurchaseCurrency>().is_err(), "{cell:?}");
        }
    }
}
