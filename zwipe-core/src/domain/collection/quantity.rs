//! How many copies of a printing a collection row holds.

use serde::{Deserialize, Serialize};

/// A count outside 1 to [`CollectionQuantity::MAX`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("must be between 1 and {max}", max = CollectionQuantity::MAX)]
pub struct InvalidCollectionQuantity;

/// Copies of one printing in a collection row: 1 to [`CollectionQuantity::MAX`]. The database
/// checks the same range, and a row never holds zero.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "i32", into = "i32")]
pub struct CollectionQuantity(i32);

impl CollectionQuantity {
    /// The most copies one row may hold. Two rows of one printing that merge in an import stop
    /// here too.
    pub const MAX: i32 = 9_999;

    /// A quantity of `copies`.
    ///
    /// # Errors
    ///
    /// Returns [`InvalidCollectionQuantity`] below 1 or above [`CollectionQuantity::MAX`].
    pub fn new(copies: i32) -> Result<Self, InvalidCollectionQuantity> {
        if (1..=Self::MAX).contains(&copies) {
            Ok(Self(copies))
        } else {
            Err(InvalidCollectionQuantity)
        }
    }

    /// The copies.
    #[must_use]
    pub fn get(self) -> i32 {
        self.0
    }

    /// This and `other` together, held at [`CollectionQuantity::MAX`].
    #[must_use]
    pub fn merged(self, other: Self) -> Self {
        Self(self.0.saturating_add(other.0).min(Self::MAX))
    }
}

impl TryFrom<i32> for CollectionQuantity {
    type Error = InvalidCollectionQuantity;

    fn try_from(copies: i32) -> Result<Self, Self::Error> {
        Self::new(copies)
    }
}

impl From<CollectionQuantity> for i32 {
    fn from(quantity: CollectionQuantity) -> Self {
        quantity.0
    }
}

/// A count outside 0 to [`CollectionQuantity::MAX`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("must be between 0 and {max}", max = CollectionQuantity::MAX)]
pub struct InvalidSetQuantity;

/// The copies a set call gives one collection row: 0 to [`CollectionQuantity::MAX`], where 0
/// removes the row.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "i32", into = "i32")]
pub struct SetQuantity(i32);

impl SetQuantity {
    /// A set call of `copies`.
    ///
    /// # Errors
    ///
    /// Returns [`InvalidSetQuantity`] below 0 or above [`CollectionQuantity::MAX`].
    pub fn new(copies: i32) -> Result<Self, InvalidSetQuantity> {
        if (0..=CollectionQuantity::MAX).contains(&copies) {
            Ok(Self(copies))
        } else {
            Err(InvalidSetQuantity)
        }
    }

    /// The copies.
    #[must_use]
    pub fn get(self) -> i32 {
        self.0
    }

    /// The row's quantity, or `None` when the call removes the row.
    #[must_use]
    pub fn row(self) -> Option<CollectionQuantity> {
        CollectionQuantity::new(self.0).ok()
    }
}

impl TryFrom<i32> for SetQuantity {
    type Error = InvalidSetQuantity;

    fn try_from(copies: i32) -> Result<Self, Self::Error> {
        Self::new(copies)
    }
}

impl From<SetQuantity> for i32 {
    fn from(quantity: SetQuantity) -> Self {
        quantity.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_set_call_takes_zero_to_the_most_and_zero_removes() {
        assert_eq!(SetQuantity::new(0).unwrap().row(), None);
        assert_eq!(
            SetQuantity::new(3).unwrap().row(),
            Some(CollectionQuantity::new(3).unwrap())
        );
        assert_eq!(SetQuantity::new(9_999).unwrap().get(), 9_999);
        assert!(SetQuantity::new(-1).is_err());
        assert!(SetQuantity::new(10_000).is_err());
        assert!(serde_json::from_str::<SetQuantity>("-1").is_err());
        assert!(serde_json::from_str::<SetQuantity>("10000").is_err());
        assert_eq!(serde_json::from_str::<SetQuantity>("0").unwrap().get(), 0);
    }

    #[test]
    fn a_row_holds_one_to_the_most() {
        assert!(CollectionQuantity::new(0).is_err());
        assert!(CollectionQuantity::new(-3).is_err());
        assert_eq!(CollectionQuantity::new(1).unwrap().get(), 1);
        assert_eq!(CollectionQuantity::new(9_999).unwrap().get(), 9_999);
        assert!(CollectionQuantity::new(10_000).is_err());
        assert!(serde_json::from_str::<CollectionQuantity>("0").is_err());
    }

    #[test]
    fn merging_stops_at_the_most() {
        let big = CollectionQuantity::new(9_000).unwrap();
        assert_eq!(big.merged(big).get(), CollectionQuantity::MAX);
        let two = CollectionQuantity::new(2).unwrap();
        assert_eq!(two.merged(two).get(), 4);
    }
}
