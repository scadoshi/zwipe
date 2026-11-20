//! The merged commander ranking as both clients ask for it: Commanders' Suggested list and every
//! commander swipe deal pages of `POST /api/commander/deal`.
//!
//! The server filters and ranks; the client only pages. A next page leaves out the commanders
//! already dealt rather than counting an offset, since a left swipe hides a commander for a week
//! and so drops it out of the ranking mid-session: an offset would then skip commanders never
//! shown. The server reads at most [`MAX_DEAL_EXCLUDE`] of them, so a session that has dealt that
//! many has no next page.

use uuid::Uuid;
use zwipe_core::domain::card::search_card::card_filter::builder::CardQueryBuilder;
use zwipe_core::domain::deck::models::format::Format;
use zwipe_core::http::contracts::suggestions::{
    HttpCommanderDeal, HttpCommanderSuggestion, MAX_DEAL_EXCLUDE, RankedBy,
};

/// The line over a list ranked by popularity: the user has nothing to rank by yet.
pub const POPULAR_LINE: &str = "Popular commanders until you've built a deck";

/// A row's reason when it shares no Oracle tag with the user's decks.
pub const POPULAR_REASON: &str = "Popular commander";

/// How many commanders a page of Commanders' Suggested list holds.
pub const LIST_PAGE: u32 = 10;

/// Where a deal comes from, which decides whether the commander maybeboard stays in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Door {
    /// Commanders' Suggested list and its Swipe: the maybeboard stays out.
    Suggested,
    /// Deck creation's commander slot, for `format`: the maybeboard stays in, since saving a
    /// commander is often why a deck gets built.
    DeckCreation(Format),
}

/// The next page for `door` under the card `filter`, `limit` commanders long, leaving out the
/// oracle ids in `dealt`; `None` once `dealt` holds [`MAX_DEAL_EXCLUDE`], past which the server
/// would deal them again.
#[must_use]
pub fn next_page(
    door: Door,
    filter: &CardQueryBuilder,
    dealt: &[Uuid],
    limit: u32,
) -> Option<HttpCommanderDeal> {
    if dealt.len() >= MAX_DEAL_EXCLUDE {
        return None;
    }
    let (format, keep_maybeboard) = match door {
        Door::Suggested => (None, false),
        Door::DeckCreation(format) => (Some(format), true),
    };
    Some(HttpCommanderDeal {
        filter: filter.build().ok(),
        format,
        offset: 0,
        limit,
        exclude: dealt.to_vec(),
        keep_maybeboard,
    })
}

/// Whether a page of `got` commanders, asked `limit` long, was the ranking's last.
#[must_use]
pub fn is_last_page(got: usize, limit: u32) -> bool {
    u32::try_from(got).is_ok_and(|got| got < limit)
}

/// The line over a list ranked by `ranked_by`: [`POPULAR_LINE`] for popularity, else none.
#[must_use]
pub fn ranked_line(ranked_by: RankedBy) -> Option<&'static str> {
    match ranked_by {
        RankedBy::Popularity => Some(POPULAR_LINE),
        RankedBy::Playstyle | RankedBy::Unknown => None,
    }
}

/// Whether `suggestion` is worded [`POPULAR_REASON`]: it shares no Oracle tag with the user's
/// decks, so popularity placed it.
#[must_use]
pub fn is_popular_only(suggestion: &HttpCommanderSuggestion) -> bool {
    suggestion.reasons.tags.is_empty()
}

/// The oracle ids of `suggestions`, for the next page to leave out.
#[must_use]
pub fn oracle_ids(suggestions: &[HttpCommanderSuggestion]) -> Vec<Uuid> {
    suggestions
        .iter()
        .filter_map(|s| s.card.scryfall_data.oracle_id)
        .collect()
}

#[cfg(test)]
mod tests {
    use zwipe_core::domain::card::models::scryfall_data::colors::Color;

    use super::*;

    fn ids(n: usize) -> Vec<Uuid> {
        (0..n).map(|_| Uuid::new_v4()).collect()
    }

    #[test]
    fn suggested_leaves_the_maybeboard_out_and_deck_creation_keeps_it_with_its_format() {
        let filter = CardQueryBuilder::new();
        let suggested = next_page(Door::Suggested, &filter, &[], LIST_PAGE).unwrap();
        assert!(!suggested.keep_maybeboard);
        assert_eq!(suggested.format, None, "Commander, the server's default");
        let deck = next_page(Door::DeckCreation(Format::Brawl), &filter, &[], 25).unwrap();
        assert!(deck.keep_maybeboard);
        assert_eq!(deck.format, Some(Format::Brawl));
        assert_eq!(deck.limit, 25);
    }

    #[test]
    fn a_next_page_leaves_out_what_was_dealt_instead_of_counting_an_offset() {
        let dealt = ids(3);
        let page = next_page(Door::Suggested, &CardQueryBuilder::new(), &dealt, LIST_PAGE).unwrap();
        assert_eq!(page.offset, 0);
        assert_eq!(page.exclude, dealt);
    }

    #[test]
    fn no_filter_sends_none_and_a_filter_goes_to_the_server() {
        let none = next_page(Door::Suggested, &CardQueryBuilder::new(), &[], LIST_PAGE).unwrap();
        assert_eq!(
            none.filter, None,
            "an empty query can't be built, so no filter is None"
        );
        let mut filter = CardQueryBuilder::new();
        filter.set_color_identity_equals(vec![Color::White, Color::Blue, Color::Red].into());
        let narrowed = next_page(Door::Suggested, &filter, &[], LIST_PAGE).unwrap();
        assert!(narrowed.filter.is_some(), "the server applies the filter");
    }

    #[test]
    fn a_session_that_dealt_the_server_s_exclude_cap_has_no_next_page() {
        let filter = CardQueryBuilder::new();
        assert!(next_page(Door::Suggested, &filter, &ids(MAX_DEAL_EXCLUDE - 1), 25).is_some());
        assert_eq!(
            next_page(Door::Suggested, &filter, &ids(MAX_DEAL_EXCLUDE), 25),
            None
        );
    }

    #[test]
    fn a_short_page_is_the_last() {
        assert!(is_last_page(3, 10));
        assert!(is_last_page(0, 10));
        assert!(!is_last_page(10, 10));
    }

    #[test]
    fn only_a_popularity_ranking_gets_the_line() {
        assert_eq!(ranked_line(RankedBy::Popularity), Some(POPULAR_LINE));
        assert_eq!(ranked_line(RankedBy::Playstyle), None);
        assert_eq!(ranked_line(RankedBy::Unknown), None);
    }
}
