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

pub mod accent_fold;
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

/// Folds accents (see [`accent_fold::fold_accents`]), then strips punctuation,
/// keeping only alphanumeric characters and whitespace, and trims the ends. Used
/// for accent- and punctuation-insensitive text search: "akromas will" matches
/// "Akroma's Will" and "jotun" matches "Jötun Grunt". The server folds the stored
/// text the same way in SQL. A card name always puts a space after a comma, so a
/// comma typed with no space after it becomes one: "yes man,personal" reads as
/// "yes man personal", as "Yes Man, Personal Securitron" strips.
#[must_use]
pub fn strip_punctuation(s: &str) -> String {
    let folded = accent_fold::fold_accents(s);
    let mut chars = folded.chars().peekable();
    let mut kept = String::with_capacity(folded.len());
    while let Some(c) = chars.next() {
        if c.is_alphanumeric() || c.is_whitespace() {
            kept.push(c);
        } else if c == ',' && chars.peek().is_some_and(|next| !next.is_whitespace()) {
            kept.push(' ');
        }
    }
    kept.trim().to_owned()
}

/// `s` with each run of whitespace made one space and the ends trimmed: the form a card name
/// is matched in, after [`strip_punctuation`]. Stripping "Fire // Ice" leaves two spaces between
/// the words, so a name search compares both sides this way and "fire ice" finds it. The server
/// collapses the stored name the same way in SQL.
#[must_use]
pub fn collapse_spaces(s: &str) -> String {
    s.split_whitespace().collect::<Vec<&str>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::{collapse_spaces, strip_punctuation};

    #[test]
    fn a_comma_with_no_space_still_splits_the_words() {
        assert_eq!(strip_punctuation("yes man,personal"), "yes man personal");
        assert_eq!(strip_punctuation("yes man, personal"), "yes man personal");
        assert_eq!(strip_punctuation("tawnos,urza's"), "tawnos urzas");
    }

    #[test]
    fn the_ends_trim_and_inner_spaces_stay_as_the_server_strips_names() {
        assert_eq!(
            strip_punctuation("  kaalia of the vast, "),
            "kaalia of the vast"
        );
        assert_eq!(
            strip_punctuation("fire // ice"),
            "fire  ice",
            "the server strips \"Fire // Ice\" to two spaces, so the query keeps them"
        );
    }

    #[test]
    fn collapsing_spaces_makes_a_split_card_s_name_read_as_typed() {
        assert_eq!(
            collapse_spaces(&strip_punctuation("Fire // Ice")),
            "Fire Ice"
        );
        assert_eq!(collapse_spaces("  fire \t  ice "), "fire ice");
        assert_eq!(collapse_spaces(""), "");
    }

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
            "AEtherize  the card"
        );
    }

    #[test]
    fn preserves_alphanumeric_and_spaces() {
        assert_eq!(strip_punctuation("Lightning Bolt"), "Lightning Bolt");
    }

    #[test]
    fn folds_accents_before_stripping() {
        assert_eq!(strip_punctuation("Lim-Dûl"), "LimDul");
        assert_eq!(strip_punctuation("Jötun"), "Jotun");
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
