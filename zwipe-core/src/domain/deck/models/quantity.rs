//! Card quantity validation for deck building.
//!
//! Ensures card quantities are valid (1-99).

use serde::{Deserialize, Serialize};
use std::ops::Deref;
use thiserror::Error;

/// Error when quantity is zero or negative.
#[derive(Debug, Error)]
#[error("must be greater than 0")]
pub struct InvalidQuantity;

/// Validated card quantity (1-99 copies).
///
/// Used when creating deck cards. Ensures quantity is positive.
/// Upper bound (deck copy limits) is enforced separately by service layer.
#[derive(Debug, Clone, PartialEq)]
pub struct Quantity(i32);

impl Quantity {
    /// Creates a new quantity with validation.
    ///
    /// # Errors
    ///
    /// Returns [`InvalidQuantity`] if quantity is less than 1.
    pub fn new(quantity: i32) -> Result<Self, InvalidQuantity> {
        if quantity < 1 {
            return Err(InvalidQuantity);
        }
        Ok(Self(quantity))
    }

    pub fn one() -> Self {
        Self(1)
    }

    pub fn four() -> Self {
        Self(4)
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
