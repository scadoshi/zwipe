//! Why a collection import is turned down as a whole. Each message is safe to show the user.

use crate::domain::collection::limits::{MAX_CELL_CHARS, MAX_ROWS};
use crate::domain::collection::tag::InvalidCollectionTags;

/// A collection import that changes nothing. A row that only fails to match a card is not one
/// of these: it lands in the reply's unmatched list and the rest import.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ImportCollectionError {
    /// No header row.
    #[error("the file is empty")]
    Empty,
    /// A header row that is neither ManaBox's export nor Archidekt's.
    #[error("this isn't a ManaBox or Archidekt collection export")]
    UnrecognizedHeader,
    /// A header and no card rows.
    #[error("the file has no cards in it")]
    NoRows,
    /// Card rows, none of which matched a card. A replace would empty the collection, so
    /// nothing changes.
    #[error("no row matched a card")]
    NothingMatched,
    /// More than [`MAX_ROWS`] card rows.
    #[error("too many rows, the limit is {MAX_ROWS}")]
    TooManyRows,
    /// A cell longer than [`MAX_CELL_CHARS`] characters.
    #[error("line {line}: a cell is over {MAX_CELL_CHARS} characters")]
    CellTooLong {
        /// The line it is on, 1 for the header.
        line: u64,
    },
    /// A line the CSV reader can't read: an unclosed quote, for one.
    #[error("line {line} isn't valid CSV")]
    Malformed {
        /// The line it is on, 1 for the header.
        line: u64,
    },
    /// A row's tags past a cap.
    #[error("line {line}: {reason}")]
    InvalidTags {
        /// The line it is on, 1 for the header.
        line: u64,
        /// Which cap.
        reason: InvalidCollectionTags,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn messages_name_the_line_and_the_cap() {
        assert_eq!(
            ImportCollectionError::TooManyRows.to_string(),
            "too many rows, the limit is 50000"
        );
        assert_eq!(
            ImportCollectionError::CellTooLong { line: 7 }.to_string(),
            "line 7: a cell is over 512 characters"
        );
        assert_eq!(
            ImportCollectionError::InvalidTags {
                line: 3,
                reason: InvalidCollectionTags::TooMany
            }
            .to_string(),
            "line 3: more than 20 tags"
        );
    }
}
