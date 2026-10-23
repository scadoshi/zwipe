//! The direction a sort runs in.

use std::fmt;

use serde::{Deserialize, Serialize};

/// Which way a sort runs.
///
/// On the wire it is its name in lowercase: `"ascending"` or `"descending"`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SortDirection {
    /// Smallest first; missing values last.
    #[default]
    Ascending,
    /// Largest first.
    Descending,
}

impl SortDirection {
    /// The other direction.
    pub fn flipped(self) -> Self {
        match self {
            Self::Ascending => Self::Descending,
            Self::Descending => Self::Ascending,
        }
    }
}

impl fmt::Display for SortDirection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ascending => write!(f, "Ascending"),
            Self::Descending => write!(f, "Descending"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::SortDirection;

    #[test]
    fn default_is_ascending() {
        assert_eq!(SortDirection::default(), SortDirection::Ascending);
    }

    #[test]
    fn flipped_swaps() {
        assert_eq!(
            SortDirection::Ascending.flipped(),
            SortDirection::Descending
        );
        assert_eq!(
            SortDirection::Descending.flipped(),
            SortDirection::Ascending
        );
    }

    #[test]
    fn wire_string_round_trip() {
        for (direction, wire) in [
            (SortDirection::Ascending, r#""ascending""#),
            (SortDirection::Descending, r#""descending""#),
        ] {
            assert_eq!(serde_json::to_string(&direction).unwrap(), wire);
            let read: SortDirection = serde_json::from_str(wire).unwrap();
            assert_eq!(read, direction);
        }
        assert!(serde_json::from_str::<SortDirection>(r#""Ascending""#).is_err());
    }

    #[test]
    fn display_matches_the_sort_sheet_labels() {
        assert_eq!(SortDirection::Ascending.to_string(), "Ascending");
        assert_eq!(SortDirection::Descending.to_string(), "Descending");
    }
}
