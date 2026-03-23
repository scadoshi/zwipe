//! Deck copy limit value object.
//!
//! Enforces MTG deck building rules for card copies (singleton vs. standard format).

use serde::{Deserialize, Serialize};
use std::ops::Deref;
use thiserror::Error;

/// Error when copy limit is neither 1 (singleton) nor 4 (standard).
#[derive(Debug, Error)]
#[error("must be standard (4) or singleton (1)")]
pub struct InvalidCopyMax;

/// Maximum copies allowed per card in a deck (1 = Commander, 4 = Standard).
///
/// # Formats
///
/// - **Singleton (1)**: Commander/EDH format - only 1 copy of each card (except basic lands)
/// - **Standard (4)**: Standard format - up to 4 copies of each card (except basic lands)
///
/// # Example
///
/// ```rust,ignore
/// let commander = CopyMax::singleton();  // 1 copy max
/// let standard = CopyMax::standard();    // 4 copies max
/// ```
#[derive(Debug, Clone, PartialEq, Copy)]
pub struct CopyMax(i32);

impl CopyMax {
    /// Creates a new copy limit with validation.
    ///
    /// # Errors
    ///
    /// Returns [`InvalidCopyMax`] if value is not 1 or 4.
    pub fn new(max: i32) -> Result<Self, InvalidCopyMax> {
        if ![1, 4].contains(&max) {
            return Err(InvalidCopyMax);
        }

        Ok(Self(max))
    }

    /// Creates a singleton format copy limit (1 copy per card).
    ///
    /// Used for Commander/EDH format decks.
    pub fn singleton() -> Self {
        Self(1)
    }

    /// Creates a standard format copy limit (4 copies per card).
    ///
    /// Used for Standard, Modern, Legacy, etc.
    pub fn standard() -> Self {
        Self(4)
    }
}

impl Deref for CopyMax {
    type Target = i32;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Serialize for CopyMax {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for CopyMax {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let max: i32 = serde::Deserialize::deserialize(deserializer)?;
        let card_copy_max = CopyMax::new(max).map_err(serde::de::Error::custom)?;
        Ok(card_copy_max)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_copy_max_new_accepts_1() {
        assert!(CopyMax::new(1).is_ok());
    }

    #[test]
    fn test_copy_max_new_accepts_4() {
        assert!(CopyMax::new(4).is_ok());
    }

    #[test]
    fn test_copy_max_new_rejects_0() {
        assert!(matches!(CopyMax::new(0), Err(InvalidCopyMax)));
    }

    #[test]
    fn test_copy_max_new_rejects_2() {
        assert!(matches!(CopyMax::new(2), Err(InvalidCopyMax)));
    }

    #[test]
    fn test_copy_max_new_rejects_5() {
        assert!(matches!(CopyMax::new(5), Err(InvalidCopyMax)));
    }

    #[test]
    fn test_copy_max_singleton_returns_1() {
        assert_eq!(*CopyMax::singleton(), 1);
    }

    #[test]
    fn test_copy_max_standard_returns_4() {
        assert_eq!(*CopyMax::standard(), 4);
    }

    #[test]
    fn test_copy_max_max_returns_inner_value() {
        assert_eq!(*CopyMax::new(1).unwrap(), 1);
        assert_eq!(*CopyMax::new(4).unwrap(), 4);
    }

    #[test]
    fn test_copy_max_serialization_round_trip() {
        let cm = CopyMax::new(4).unwrap();
        let serialized = serde_json::to_value(cm).unwrap();
        let deserialized = serde_json::from_value::<CopyMax>(serialized).unwrap();
        assert_eq!(*deserialized, 4);
    }
}
