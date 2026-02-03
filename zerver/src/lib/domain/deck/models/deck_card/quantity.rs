//! Card quantity validation for deck building.
//!
//! Ensures card quantities are valid (1-99) and enforces deck copy limits:
//! - **Singleton decks** (Commander format): 1 copy max per card (except basic lands)
//! - **Standard decks**: 4 copies max per card (except basic lands)
//! - **Basic lands**: Unlimited (up to 99) in any format
//!
//! Copy limit enforcement happens at the service layer where deck copy_max
//! is known. This module only validates that quantities are positive.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Error when quantity is zero or negative.
#[derive(Debug, Error)]
#[error("must be greater than 0")]
pub struct InvalidQuantity;

/// Error when update quantity is zero (would have no effect).
#[derive(Debug, Error)]
#[error("must be less or greater than 0")]
pub struct InvalidUpdateQuanity;

/// Validated card quantity (1-99 copies).
///
/// Used when creating deck cards. Ensures quantity is positive.
/// Upper bound (deck copy limits) is enforced separately by service layer.
#[derive(Debug, Clone)]
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

    /// Returns the quantity value.
    pub fn quantity(&self) -> i32 {
        self.0
    }
}

impl Serialize for Quantity {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.quantity().serialize(serializer)
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

/// Delta quantity for updating card counts (can be negative to remove copies).
///
/// Used when updating existing deck cards. Allows positive (add copies)
/// or negative (remove copies) values, but not zero (would have no effect).
///
/// # Example
///
/// ```rust,ignore
/// // Add 2 more copies
/// let update = UpdateQuantity::new(2)?;
///
/// // Remove 1 copy
/// let update = UpdateQuantity::new(-1)?;
/// ```
#[derive(Debug, Clone)]
pub struct UpdateQuantity(i32);

impl UpdateQuantity {
    /// Creates a new update quantity with validation.
    ///
    /// # Errors
    ///
    /// Returns [`InvalidUpdateQuanity`] if quantity is zero (no-op).
    pub fn new(add_quantity: i32) -> Result<Self, InvalidUpdateQuanity> {
        if add_quantity == 0 {
            return Err(InvalidUpdateQuanity);
        }
        Ok(Self(add_quantity))
    }

    /// Returns the delta value (positive to add, negative to remove).
    pub fn value(&self) -> i32 {
        self.0
    }
}
