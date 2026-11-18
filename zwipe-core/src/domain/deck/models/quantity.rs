//! Card quantity validation for deck building.
//!
//! A deck card holds 1 to 99 copies of a card.

use std::ops::Deref;

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Error when a quantity is outside 1 to [`Quantity::MAX`].
#[derive(Debug, Error)]
#[error("must be between 1 and 99")]
pub struct InvalidQuantity;

/// Validated card quantity, 1 to [`Quantity::MAX`] copies: in a request, in a stored row (the
/// database checks the same range) and in a reply. The deck's total card limit is the server's
/// to enforce.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Quantity(i32);

impl Quantity {
    /// The most copies of one card a request may set.
    pub const MAX: i32 = 99;

    /// Creates a new quantity with validation.
    ///
    /// # Errors
    ///
    /// Returns [`InvalidQuantity`] if `quantity` is below 1 or above [`Quantity::MAX`].
    pub fn new(quantity: i32) -> Result<Self, InvalidQuantity> {
        if !(1..=Self::MAX).contains(&quantity) {
            return Err(InvalidQuantity);
        }
        Ok(Self(quantity))
    }

    /// A single copy.
    #[must_use]
    pub fn one() -> Self {
        Self(1)
    }

    /// A full playset.
    #[must_use]
    pub fn four() -> Self {
        Self(4)
    }
}

impl TryFrom<i32> for Quantity {
    type Error = InvalidQuantity;

    fn try_from(quantity: i32) -> Result<Self, Self::Error> {
        Self::new(quantity)
    }
}

impl Deref for Quantity {
    type Target = i32;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Serialize for Quantity {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Quantity {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let quantity = i32::deserialize(deserializer)?;
        Quantity::new(quantity).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Quantity ---

    #[test]
    fn test_quantity_new_accepts_1() {
        assert!(Quantity::new(1).is_ok());
    }

    #[test]
    fn test_quantity_new_accepts_large_value() {
        assert!(Quantity::new(99).is_ok());
    }

    #[test]
    fn test_quantity_new_rejects_0() {
        assert!(matches!(Quantity::new(0), Err(InvalidQuantity)));
    }

    #[test]
    fn test_quantity_new_rejects_negative() {
        assert!(matches!(Quantity::new(-1), Err(InvalidQuantity)));
    }

    #[test]
    fn a_request_holds_at_most_99_copies() {
        assert_eq!(Quantity::MAX, 99);
        assert!(Quantity::new(99).is_ok());
        assert!(matches!(Quantity::new(100), Err(InvalidQuantity)));
        assert_eq!(InvalidQuantity.to_string(), "must be between 1 and 99");
    }

    #[test]
    fn a_raw_count_converts_through_the_same_check() {
        assert_eq!(Quantity::try_from(99).map(|q| *q).ok(), Some(99));
        assert!(Quantity::try_from(100).is_err());
        assert!(Quantity::try_from(0).is_err());
    }

    #[test]
    fn a_count_outside_1_to_99_does_not_parse() {
        assert!(serde_json::from_value::<Quantity>(serde_json::json!(100)).is_err());
        assert!(serde_json::from_value::<Quantity>(serde_json::json!(0)).is_err());
    }

    #[test]
    fn test_quantity_value_returns_inner() {
        assert_eq!(*Quantity::new(1).unwrap(), 1);
    }

    #[test]
    fn test_quantity_serialization_round_trip() {
        let quantity = Quantity::new(1).unwrap();
        let serialized = serde_json::to_value(quantity).unwrap();
        let deserialized = serde_json::from_value::<Quantity>(serialized).unwrap();
        assert_eq!(*deserialized, 1);
    }
}
