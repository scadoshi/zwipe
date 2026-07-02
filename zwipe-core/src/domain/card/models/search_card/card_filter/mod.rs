//! Card search criteria and query construction.
//!
//! Two types share one predicate core, [`CardCriteria`]:
//!
//! - [`CardQuery`] — the **database query** POSTed to the server: criteria plus
//!   bounded pagination ([`query::Limit`]) and ordering.
//! - [`Cards`](crate::domain::card::search_card::cards::Cards) — the
//!   **in-memory collection**, whose operations take bare criteria and cannot
//!   express a limit.
//!
//! Both are built from one [`CardQueryBuilder`](builder::CardQueryBuilder):
//! `build()` for the server path, `build_criteria()` for the in-memory path.

/// Builder producing [`CardQuery`] / [`CardCriteria`] with validation.
pub mod builder;
/// Sort key options (name, CMC, rarity, etc.).
pub mod card_sort_key;
/// The shared predicate core (~50 criteria fields) and `matches()`.
pub mod criteria;
/// Criteria validation errors.
pub mod error;
/// Currency selector for the price-range filter.
pub mod price_currency;
/// The server search request: criteria + `Limit` + offset + sort.
pub mod query;

pub use card_sort_key::CardSortKey;
pub use criteria::CardCriteria;
pub use query::{CardQuery, Limit};

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
