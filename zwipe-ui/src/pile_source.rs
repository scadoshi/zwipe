//! The From chip both swipe screens share, Add and the commander swipe: which pile they deal.
//!
//! Search deals the screen's own search, Maybe the maybeboard, and Collection only cards the
//! user owns. Collection shows only while the user has a collection, as the collection summary
//! says; without one the chip cycles Search and Maybe alone, and a pile left on Collection
//! falls back to Search.

use zwipe_core::domain::card::search_card::card_filter::builder::CardQueryBuilder;

/// The toast when the collection turns out empty while From is on Collection, and Search takes
/// over.
pub const COLLECTION_NOTHING: &str = "Nothing from your collection to deal, switched to Search";

/// Which pile a swipe screen deals.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Source {
    /// The screen's search, the pick on entry.
    #[default]
    Search,
    /// The maybeboard.
    Maybeboard,
    /// Cards the user owns, the screen's search narrowed to the collection.
    Collection,
}

/// Whether the user has a collection, as the collection summary says, which decides whether
/// [`Source::Collection`] is on offer.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Holdings {
    /// Nothing known to be held: an empty collection, or no summary yet, a failed one or an
    /// older server.
    #[default]
    Empty,
    /// The collection holds something.
    Held,
}

impl Holdings {
    /// The holdings for whether the user `has_collection`.
    #[must_use]
    pub const fn from_has_collection(has_collection: bool) -> Self {
        if has_collection {
            Self::Held
        } else {
            Self::Empty
        }
    }

    /// Whether the user has a collection.
    #[must_use]
    pub const fn has_collection(self) -> bool {
        matches!(self, Self::Held)
    }
}

impl Source {
    /// Every source, in the order the chip cycles through them.
    pub const ALL: [Self; 3] = [Self::Search, Self::Maybeboard, Self::Collection];

    /// The chip's words.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Search => "Search",
            Self::Maybeboard => "Maybe",
            Self::Collection => "Collection",
        }
    }

    /// Whether the source is on offer: Collection only while the user `has_collection`.
    #[must_use]
    pub fn offered(self, has_collection: bool) -> bool {
        match self {
            Self::Search | Self::Maybeboard => true,
            Self::Collection => has_collection,
        }
    }

    /// The source a tap on the chip moves to: the next one on offer, Search after the last.
    #[must_use]
    pub fn next(self, has_collection: bool) -> Self {
        let next = match self {
            Self::Search => Self::Maybeboard,
            Self::Maybeboard => Self::Collection,
            Self::Collection => Self::Search,
        };
        if next.offered(has_collection) {
            next
        } else {
            next.next(has_collection)
        }
    }

    /// Whether the source deals a search, so it pages from the server: Search and Collection,
    /// which is the same search narrowed to owned cards.
    #[must_use]
    pub fn searches(self) -> bool {
        matches!(self, Self::Search | Self::Collection)
    }

    /// Whether a first page of `dealt` cards, under the user's `filter`, has dealt the collection
    /// dry: Collection with no filter of the user's deals every owned card the screen can take, so
    /// nothing means none is left (skipped, none fits) or the collection is gone, which only a
    /// fresh summary can tell.
    #[must_use]
    pub fn ran_dry(self, filter: &CardQueryBuilder, dealt: usize) -> bool {
        dealt == 0 && self == Self::Collection && filter.is_empty()
    }

    /// Whether the source's search is narrowed to owned cards.
    #[must_use]
    pub fn owned_only(self) -> bool {
        self == Self::Collection
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_chip_cycles_search_maybe_collection_and_back() {
        assert_eq!(Source::Search.next(true), Source::Maybeboard);
        assert_eq!(Source::Maybeboard.next(true), Source::Collection);
        assert_eq!(Source::Collection.next(true), Source::Search);
    }

    #[test]
    fn without_a_collection_the_chip_skips_collection() {
        assert_eq!(Source::Search.next(false), Source::Maybeboard);
        assert_eq!(Source::Maybeboard.next(false), Source::Search);
        assert!(!Source::Collection.offered(false));
        assert!(Source::Collection.offered(true));
    }

    #[test]
    fn collection_searches_owned_cards_and_maybe_deals_no_search() {
        assert!(Source::Collection.searches() && Source::Collection.owned_only());
        assert!(Source::Search.searches() && !Source::Search.owned_only());
        assert!(!Source::Maybeboard.searches() && !Source::Maybeboard.owned_only());
    }

    #[test]
    fn only_an_unfiltered_collection_that_dealt_nothing_runs_dry() {
        let blank = CardQueryBuilder::default();
        assert!(Source::Collection.ran_dry(&blank, 0));
        assert!(!Source::Collection.ran_dry(&blank, 3), "it dealt cards");
        assert!(
            !Source::Search.ran_dry(&blank, 0),
            "Search keeps its empty pile"
        );
        let mut named = CardQueryBuilder::default();
        named.set_name_contains("bolt");
        assert!(
            !Source::Collection.ran_dry(&named, 0),
            "the user's filter is why"
        );
    }

    #[test]
    fn the_chip_reads_search_maybe_and_collection() {
        let labels: Vec<&str> = Source::ALL.iter().map(|s| s.label()).collect();
        assert_eq!(labels, ["Search", "Maybe", "Collection"]);
    }
}
