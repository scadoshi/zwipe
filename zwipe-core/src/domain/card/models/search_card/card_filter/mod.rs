//! Card search criteria and query construction.
//!
//! Two types share one predicate core, [`CardCriteria`]:
//!
//! - [`CardQuery`]: the **database query** sent to the server in a POST: criteria plus
//!   bounded pagination ([`Limit`]) and ordering.
//! - [`Cards`](crate::domain::card::search_card::cards::Cards): the
//!   **in-memory collection**, whose operations take bare criteria and cannot
//!   express a limit.
//!
//! Both are built from one [`CardQueryBuilder`](crate::domain::card::models::search_card::card_filter::builder::CardQueryBuilder):
//! `build()` for the server path, `build_criteria()` for the in-memory path.
//!
//! On the wire a [`CardQuery`] is the criteria flattened to the top level
//! beside `limit` / `offset` / `order_by` / `direction` / `synergy` /
//! `include_skipped`. `direction` is `"ascending"` or `"descending"` and
//! defaults to ascending when absent.

pub mod builder;
pub mod card_sort_key;
pub mod criteria;
pub mod error;
pub mod price_currency;
pub mod query;
pub mod sort_direction;

pub use card_sort_key::CardSortKey;
pub use criteria::CardCriteria;
pub use query::{CardQuery, Limit};
pub use sort_direction::SortDirection;

/// Strips punctuation from a string, keeping only alphanumeric characters and whitespace.
/// Used for punctuation-insensitive text search (e.g., "akromas will" matches "Akroma's Will").
pub fn strip_punctuation(s: &str) -> String {
    s.chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::strip_punctuation;

    #[test]
    fn strips_apostrophes() {
        assert_eq!(strip_punctuation("Akroma's Will"), "Akromas Will");
    }

    #[test]
    fn strips_commas() {
        assert_eq!(
            strip_punctuation("Satya, Aetherflux Genius"),
            "Satya Aetherflux Genius"
        );
    }

    #[test]
    fn strips_multiple_punctuation() {
        assert_eq!(
            strip_punctuation("Ætherize — the card!"),
            "Ætherize  the card"
        );
    }

    #[test]
    fn preserves_alphanumeric_and_spaces() {
        assert_eq!(strip_punctuation("Lightning Bolt"), "Lightning Bolt");
    }

    #[test]
    fn empty_string() {
        assert_eq!(strip_punctuation(""), "");
    }

    #[test]
    fn only_punctuation() {
        assert_eq!(strip_punctuation("!@#$%"), "");
    }
}
