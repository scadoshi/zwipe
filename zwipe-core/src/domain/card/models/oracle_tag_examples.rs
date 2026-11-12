//! An oracle tag's example cards, as the public examples route serves them: the most played
//! cards carrying the tag, a page at a time, tokens left out.
//!
//! The route takes the tag as a path segment, so the slug is checked before it reaches a query:
//! lowercase letters, digits and hyphens, starting with a letter or digit, at most
//! [`MAX_SLUG_LEN`] long. Every slug in Scryfall's catalog fits.

use std::fmt;
use std::str::FromStr;

use thiserror::Error;

use crate::domain::card::search_card::card_filter::CardQuery;
use crate::domain::card::search_card::card_filter::builder::CardQueryBuilder;
use crate::domain::card::search_card::card_filter::card_sort_key::CardSortKey;
use crate::domain::card::search_card::card_filter::error::InvalidCardCriteria;

/// The longest slug the route accepts. Scryfall's longest is 34.
pub const MAX_SLUG_LEN: usize = 64;

/// How many example cards one page holds: the server always serves this many or fewer.
pub const EXAMPLES_PAGE_SIZE: u32 = 25;

/// The furthest offset the route serves from: the examples run to 500 cards.
pub const EXAMPLES_MAX_OFFSET: u32 = 500;

/// An oracle tag slug that passed the route's check, such as `spot-removal`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OracleTagSlug(String);

/// Why a string is not an oracle tag slug.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum InvalidOracleTagSlug {
    /// The slug is empty.
    #[error("oracle tag slug is empty")]
    Empty,
    /// The slug is longer than [`MAX_SLUG_LEN`].
    #[error("oracle tag slug is longer than {MAX_SLUG_LEN} characters")]
    TooLong,
    /// The slug has a character other than a lowercase letter, a digit or a hyphen, or starts
    /// with a hyphen.
    #[error("oracle tag slug has a character other than a-z, 0-9 or an inner hyphen")]
    BadCharacter,
}

impl FromStr for OracleTagSlug {
    type Err = InvalidOracleTagSlug;

    fn from_str(slug: &str) -> Result<Self, Self::Err> {
        let Some(first) = slug.chars().next() else {
            return Err(InvalidOracleTagSlug::Empty);
        };
        if slug.len() > MAX_SLUG_LEN {
            return Err(InvalidOracleTagSlug::TooLong);
        }
        let allowed = |c: char| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-';
        if first == '-' || !slug.chars().all(allowed) {
            return Err(InvalidOracleTagSlug::BadCharacter);
        }
        Ok(Self(slug.to_owned()))
    }
}

impl OracleTagSlug {
    /// The slug as written.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for OracleTagSlug {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Why an examples page can't be served.
#[derive(Debug, PartialEq, Error)]
pub enum InvalidExamplesPage {
    /// The offset is past [`EXAMPLES_MAX_OFFSET`].
    #[error("offset {0} is past the examples' last page at {EXAMPLES_MAX_OFFSET}")]
    PastTheEnd(u32),
    /// The search the page builds is invalid.
    #[error(transparent)]
    Query(#[from] InvalidCardCriteria),
}

/// The search for `slug`'s examples from `offset`: the cards carrying the tag, no tokens, most
/// played first, [`EXAMPLES_PAGE_SIZE`] of them.
///
/// # Errors
///
/// [`InvalidExamplesPage::PastTheEnd`] for an offset past [`EXAMPLES_MAX_OFFSET`], and
/// [`InvalidExamplesPage::Query`] if the search does not build.
pub fn examples_query(slug: &OracleTagSlug, offset: u32) -> Result<CardQuery, InvalidExamplesPage> {
    if offset > EXAMPLES_MAX_OFFSET {
        return Err(InvalidExamplesPage::PastTheEnd(offset));
    }
    let mut builder = CardQueryBuilder::new();
    builder
        .set_oracle_tags_contains_any(vec![slug.as_str().to_owned()])
        .set_is_token(false)
        .set_sort(CardSortKey::EdhrecRank)
        .set_limit(EXAMPLES_PAGE_SIZE)
        .set_offset(offset);
    Ok(builder.build()?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_catalog_slug_passes() {
        for slug in ["spot-removal", "typal-goblin", "burn", "cast-trigger-2"] {
            let parsed: OracleTagSlug = slug.parse().unwrap();
            assert_eq!(parsed.as_str(), slug);
        }
    }

    #[test]
    fn anything_else_is_turned_away() {
        assert_eq!(
            "".parse::<OracleTagSlug>(),
            Err(InvalidOracleTagSlug::Empty)
        );
        assert_eq!(
            "a".repeat(MAX_SLUG_LEN + 1).parse::<OracleTagSlug>(),
            Err(InvalidOracleTagSlug::TooLong)
        );
        for bad in [
            "Spot-Removal",
            "-burn",
            "burn removal",
            "burn%27",
            "burn'or'1",
            "../x",
            "ünicode",
        ] {
            assert_eq!(
                bad.parse::<OracleTagSlug>(),
                Err(InvalidOracleTagSlug::BadCharacter),
                "{bad}"
            );
        }
        assert!("a".repeat(MAX_SLUG_LEN).parse::<OracleTagSlug>().is_ok());
    }

    #[test]
    fn a_page_is_capped_and_ends_at_the_last_offset() {
        let slug: OracleTagSlug = "burn".parse().unwrap();
        let query = examples_query(&slug, 0).unwrap();
        assert_eq!(query.limit(), EXAMPLES_PAGE_SIZE);
        assert!(examples_query(&slug, EXAMPLES_MAX_OFFSET).is_ok());
        assert_eq!(
            examples_query(&slug, EXAMPLES_MAX_OFFSET + 1),
            Err(InvalidExamplesPage::PastTheEnd(EXAMPLES_MAX_OFFSET + 1))
        );
    }
}
