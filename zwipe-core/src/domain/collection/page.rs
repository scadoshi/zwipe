//! Reading the collection a page at a time: what a search may say and how long a page runs.

use serde::{Deserialize, Serialize};

use crate::domain::collection::tag::strip_unseen;

/// A name search longer than [`CollectionSearch::MAX_CHARS`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("a search is at most {max} characters", max = CollectionSearch::MAX_CHARS)]
pub struct InvalidCollectionSearch;

/// The text a collection list searches card names for: at most [`CollectionSearch::MAX_CHARS`]
/// characters, with characters that draw nothing (controls, zero-width marks) dropped and the
/// ends trimmed. Empty means no search.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct CollectionSearch(String);

impl CollectionSearch {
    /// The longest search, in characters, counted before anything is dropped.
    pub const MAX_CHARS: usize = 100;

    /// The search for `raw`.
    ///
    /// # Errors
    ///
    /// Returns [`InvalidCollectionSearch`] past [`CollectionSearch::MAX_CHARS`] characters.
    pub fn new(raw: &str) -> Result<Self, InvalidCollectionSearch> {
        if raw.chars().count() > Self::MAX_CHARS {
            return Err(InvalidCollectionSearch);
        }
        Ok(Self(strip_unseen(raw).trim().to_owned()))
    }

    /// The search text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Whether the search is empty, so the list holds every row.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl TryFrom<String> for CollectionSearch {
    type Error = InvalidCollectionSearch;

    fn try_from(raw: String) -> Result<Self, Self::Error> {
        Self::new(&raw)
    }
}

impl From<CollectionSearch> for String {
    fn from(search: CollectionSearch) -> Self {
        search.0
    }
}

/// A page size outside 1 to [`CollectionPageSize::MAX`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("a page holds 1 to {max} rows", max = CollectionPageSize::MAX)]
pub struct InvalidCollectionPageSize;

/// How many rows one page of the collection list holds: 1 to [`CollectionPageSize::MAX`],
/// [`CollectionPageSize::DEFAULT`] when a call doesn't say.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "u32", into = "u32")]
pub struct CollectionPageSize(u32);

impl CollectionPageSize {
    /// The longest page.
    pub const MAX: u32 = 100;
    /// The page a call that doesn't say gets.
    pub const DEFAULT: Self = Self(50);

    /// A page of `rows`.
    ///
    /// # Errors
    ///
    /// Returns [`InvalidCollectionPageSize`] for 0 or past [`CollectionPageSize::MAX`].
    pub fn new(rows: u32) -> Result<Self, InvalidCollectionPageSize> {
        if (1..=Self::MAX).contains(&rows) {
            Ok(Self(rows))
        } else {
            Err(InvalidCollectionPageSize)
        }
    }

    /// The rows.
    #[must_use]
    pub fn get(self) -> u32 {
        self.0
    }
}

impl Default for CollectionPageSize {
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl TryFrom<u32> for CollectionPageSize {
    type Error = InvalidCollectionPageSize;

    fn try_from(rows: u32) -> Result<Self, Self::Error> {
        Self::new(rows)
    }
}

impl From<CollectionPageSize> for u32 {
    fn from(size: CollectionPageSize) -> Self {
        size.0
    }
}

/// Text that is not a collection cursor: empty, past [`CollectionCursor::MAX_CHARS`], or holding
/// anything but lowercase hex digits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("not a collection cursor")]
pub struct InvalidCollectionCursor;

/// Where the next page of a collection list starts: the server's mark after the last row it sent,
/// opaque to a client, which only hands it back. Lowercase hex, 1 to
/// [`CollectionCursor::MAX_CHARS`] characters.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct CollectionCursor(String);

impl CollectionCursor {
    /// The longest cursor.
    pub const MAX_CHARS: usize = 2048;

    /// The cursor `text`.
    ///
    /// # Errors
    ///
    /// Returns [`InvalidCollectionCursor`] for empty text, text past
    /// [`CollectionCursor::MAX_CHARS`], or any character but `0`-`9` and `a`-`f`.
    pub fn new(text: &str) -> Result<Self, InvalidCollectionCursor> {
        let hex = text
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b));
        if text.is_empty() || text.len() > Self::MAX_CHARS || !hex {
            return Err(InvalidCollectionCursor);
        }
        Ok(Self(text.to_owned()))
    }

    /// The cursor's text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for CollectionCursor {
    type Error = InvalidCollectionCursor;

    fn try_from(text: String) -> Result<Self, Self::Error> {
        Self::new(&text)
    }
}

impl From<CollectionCursor> for String {
    fn from(cursor: CollectionCursor) -> Self {
        cursor.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_cursor_is_lowercase_hex_within_its_cap() {
        assert!(CollectionCursor::new("0a1b2c").is_ok());
        for bad in ["", "0A", "xyz", "12 34", "&offset=1"] {
            assert!(CollectionCursor::new(bad).is_err(), "{bad:?}");
        }
        assert!(CollectionCursor::new(&"a".repeat(2048)).is_ok());
        assert!(CollectionCursor::new(&"a".repeat(2049)).is_err());
        assert!(serde_json::from_str::<CollectionCursor>("\"ZZ\"").is_err());
    }

    #[test]
    fn a_search_drops_what_draws_nothing_and_caps_its_length() {
        assert_eq!(
            CollectionSearch::new("  Sol\u{200b} Ring\u{0} ")
                .unwrap()
                .as_str(),
            "Sol Ring"
        );
        assert!(CollectionSearch::new("").unwrap().is_empty());
        assert!(CollectionSearch::new(&"a".repeat(100)).is_ok());
        assert!(CollectionSearch::new(&"a".repeat(101)).is_err());
        assert!(
            serde_json::from_str::<CollectionSearch>(&format!("\"{}\"", "é".repeat(101))).is_err()
        );
        let sql = "'; DROP TABLE collection_cards; --";
        assert_eq!(
            CollectionSearch::new(sql).unwrap().as_str(),
            sql,
            "text, kept as typed"
        );
    }

    #[test]
    fn a_page_holds_one_to_the_most() {
        assert!(CollectionPageSize::new(0).is_err());
        assert_eq!(CollectionPageSize::new(1).unwrap().get(), 1);
        assert_eq!(CollectionPageSize::new(100).unwrap().get(), 100);
        assert!(CollectionPageSize::new(101).is_err());
        assert_eq!(CollectionPageSize::default().get(), 50);
        assert!(serde_json::from_str::<CollectionPageSize>("1000").is_err());
    }
}
