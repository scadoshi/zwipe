//! The merged commander ranking as both clients ask for it: Commanders' Suggested list and every
//! commander swipe deal pages of `POST /api/commander/deal`.
//!
//! The server filters and ranks; the client only pages. A next page leaves out the commanders
//! already dealt rather than counting an offset, since a left swipe hides a commander for a week
//! and so drops it out of the ranking mid-session: an offset would then skip commanders never
//! shown. The server reads at most [`MAX_DEAL_EXCLUDE`] of them, so a session that has dealt that
//! many has no next page.

use std::cmp::Ordering;

use uuid::Uuid;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::models::scryfall_data::ScryfallData;
use zwipe_core::domain::card::search_card::card_filter::builder::CardQueryBuilder;
use zwipe_core::domain::card::search_card::card_filter::card_sort_key::CardSortKey;
use zwipe_core::domain::card::search_card::card_filter::sort_direction::SortDirection;
use zwipe_core::domain::card::search_card::cards::Cards;
use zwipe_core::domain::deck::models::format::Format;
use zwipe_core::http::contracts::suggestions::{
    HttpCommanderDeal, HttpCommanderSuggestion, MAX_DEAL_EXCLUDE, RankedBy,
};

use crate::deck::form;

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

/// `filter` as `door` deals under it: deck creation narrows it as every commander pick does
/// ([`form::narrow_to_commanders`]), to its format's commander rules and its legality, joined to
/// any legality the filter already asks for. Suggested has no deck format and takes `filter` as it
/// is.
#[must_use]
pub fn door_filter(door: Door, filter: &CardQueryBuilder) -> CardQueryBuilder {
    let mut dealt = filter.clone();
    if let Door::DeckCreation(format) = door {
        form::narrow_to_commanders(&mut dealt, format);
    }
    dealt
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
        filter: door_filter(door, filter).build().ok(),
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

/// `saved` narrowed to the commanders that lead a deck in `door`'s format (Commander for
/// Suggested) by the format's commander rules, are legal in it where the door has a deck format,
/// as the deal asks the server for, and that `filter` matches. A filter that doesn't build
/// narrows by the format alone.
#[must_use]
pub fn maybeboard_matching(saved: Vec<Card>, filter: &CardQueryBuilder, door: Door) -> Vec<Card> {
    let format = match door {
        Door::Suggested => Format::Commander,
        Door::DeckCreation(format) => format,
    };
    let mut narrowed = door_filter(door, filter);
    narrowed.set_is_commander_in_format(format);
    let criteria = narrowed.build_criteria().or_else(|_| {
        let mut format_only = door_filter(door, &CardQueryBuilder::new());
        format_only.set_is_commander_in_format(format);
        format_only.build_criteria()
    });
    let matched: Vec<Card> = match criteria {
        Ok(criteria) => Cards::from(saved).matching(&criteria).into(),
        Err(_) => saved,
    };
    deal_order(matched, filter.sort().map(|key| (key, filter.direction())))
}

/// `cards` in the order the deal serves `sort` in, for the Maybe pile the device filters itself:
/// the cards the deal leaves out for the key are out (a power or toughness that isn't a number, a
/// missing or empty price), unranked cards sort last both ways under EDHREC rank and best match,
/// best match is best first whatever the direction, and a tie goes by name, then oracle id.
/// Random shuffles. No sort keeps `cards` as they are.
///
/// The server's best match also lifts the 100 most-helmed commanders by their deck count, which
/// a saved card doesn't carry, so here it is the EDHREC rank alone. Names compare by their bytes,
/// not by the server's collation.
#[must_use]
pub fn deal_order(cards: Vec<Card>, sort: Option<(CardSortKey, SortDirection)>) -> Vec<Card> {
    let Some((key, direction)) = sort else {
        return cards;
    };
    let mut kept: Vec<Card> = cards
        .into_iter()
        .filter(|card| deal_keeps(key, &card.scryfall_data))
        .collect();
    if key == CardSortKey::Random {
        return Cards::from(kept).sorted(key, direction).into();
    }
    kept.sort_by(|a, b| {
        let (a, b) = (&a.scryfall_data, &b.scryfall_data);
        deal_key_order(key, direction, a, b)
            .then_with(|| a.name.cmp(&b.name))
            .then_with(|| a.oracle_id.cmp(&b.oracle_id))
    });
    kept
}

/// Whether the deal keeps `data` under `key`: a stat sort needs a whole number, a price sort a
/// price, as the server's sort filters ask.
fn deal_keeps(key: CardSortKey, data: &ScryfallData) -> bool {
    let whole =
        |v: Option<&str>| v.is_some_and(|s| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit()));
    let priced = |v: Option<&str>| v.is_some_and(|s| !s.is_empty());
    match key {
        CardSortKey::Power => whole(data.power.as_deref()),
        CardSortKey::Toughness => whole(data.toughness.as_deref()),
        CardSortKey::PriceUsd => priced(data.prices.usd.as_deref()),
        CardSortKey::PriceEur => priced(data.prices.eur.as_deref()),
        CardSortKey::PriceTix => priced(data.prices.tix.as_deref()),
        CardSortKey::Name
        | CardSortKey::Cmc
        | CardSortKey::Rarity
        | CardSortKey::ReleasedAt
        | CardSortKey::EdhrecRank
        | CardSortKey::Random
        | CardSortKey::BestMatch => true,
    }
}

/// `a` against `b` on `key` alone, as the deal orders it.
fn deal_key_order(
    key: CardSortKey,
    direction: SortDirection,
    a: &ScryfallData,
    b: &ScryfallData,
) -> Ordering {
    let directed = |ord: Ordering, direction: SortDirection| match direction {
        SortDirection::Ascending => ord,
        SortDirection::Descending => ord.reverse(),
    };
    match key {
        CardSortKey::EdhrecRank | CardSortKey::BestMatch => {
            let direction = if key == CardSortKey::BestMatch {
                SortDirection::Ascending
            } else {
                direction
            };
            match (a.edhrec_rank, b.edhrec_rank) {
                (Some(a), Some(b)) => directed(a.cmp(&b), direction),
                (Some(_), None) => Ordering::Less,
                (None, Some(_)) => Ordering::Greater,
                (None, None) => Ordering::Equal,
            }
        }
        CardSortKey::Name
        | CardSortKey::Cmc
        | CardSortKey::Power
        | CardSortKey::Toughness
        | CardSortKey::Rarity
        | CardSortKey::ReleasedAt
        | CardSortKey::PriceUsd
        | CardSortKey::PriceEur
        | CardSortKey::PriceTix
        | CardSortKey::Random => directed(key.compare(a, b), direction),
    }
}

#[cfg(test)]
mod tests {
    use zwipe_core::domain::card::models::scryfall_data::colors::Color;
    use zwipe_core::domain::card::models::scryfall_data::rarity::Rarity;

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
    fn deck_creation_deals_only_commanders_legal_in_its_format() {
        let mut filter = CardQueryBuilder::new();
        filter.set_legalities_contains_all(vec!["commander".to_owned()]);
        let brawl = door_filter(Door::DeckCreation(Format::Brawl), &filter);
        assert_eq!(
            brawl.legalities_contains_all(),
            Some(["commander".to_owned(), "brawl".to_owned()].as_slice()),
            "the format joins what the user asked for"
        );
        let page = next_page(
            Door::DeckCreation(Format::StandardBrawl),
            &CardQueryBuilder::new(),
            &[],
            25,
        )
        .unwrap();
        assert_eq!(
            page.filter.and_then(|q| q
                .criteria()
                .legalities_contains_all()
                .map(<[String]>::to_vec)),
            Some(vec!["standardbrawl".to_owned()]),
            "the server deals in the format's legality"
        );
        assert_eq!(
            door_filter(Door::Suggested, &CardQueryBuilder::new()).legalities_contains_all(),
            None,
            "Suggested has no deck format"
        );
        assert_eq!(
            brawl.is_commander_in_format(),
            Some(&Format::Brawl),
            "the same narrowing as every other commander pick (form::narrow_to_commanders)"
        );
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

    /// A commander named `name`, legal in Commander.
    fn commander(name: &str) -> Card {
        let mut card = crate::test_support::card(name);
        card.scryfall_data.type_line = Some("Legendary Creature — Goblin".to_owned());
        card.scryfall_data.power = Some("1".to_owned());
        card.scryfall_data.toughness = Some("1".to_owned());
        card
    }

    fn names(cards: &[Card]) -> Vec<&str> {
        cards
            .iter()
            .map(|c| c.scryfall_data.name.as_str())
            .collect()
    }

    #[test]
    fn the_maybe_pile_takes_the_filter_s_sort_both_ways() {
        let rarities = [
            ("Mythic", Rarity::Mythic),
            ("Common", Rarity::Common),
            ("Special", Rarity::Special),
            ("Rare", Rarity::Rare),
            ("Uncommon", Rarity::Uncommon),
        ];
        let saved: Vec<Card> = rarities
            .iter()
            .map(|(name, rarity)| {
                let mut card = commander(name);
                card.scryfall_data.rarity = *rarity;
                card
            })
            .collect();
        let mut filter = CardQueryBuilder::new();
        filter.set_sort(CardSortKey::Rarity);
        let door = Door::DeckCreation(Format::Commander);
        let up = maybeboard_matching(saved.clone(), &filter, door);
        assert_eq!(
            names(&up),
            ["Common", "Uncommon", "Rare", "Mythic", "Special"]
        );
        filter.set_direction(SortDirection::Descending);
        let down = maybeboard_matching(saved.clone(), &filter, door);
        assert_eq!(
            names(&down),
            ["Special", "Mythic", "Rare", "Uncommon", "Common"]
        );
        let unsorted = maybeboard_matching(saved, &CardQueryBuilder::new(), door);
        assert_eq!(
            names(&unsorted),
            ["Mythic", "Common", "Special", "Rare", "Uncommon"],
            "no sort keeps the saved order"
        );
    }

    #[test]
    fn a_stat_or_price_sort_leaves_out_what_the_deal_leaves_out() {
        let with = |name: &str, power: Option<&str>, usd: Option<&str>| {
            let mut card = crate::test_support::card(name);
            card.scryfall_data.power = power.map(str::to_owned);
            card.scryfall_data.prices.usd = usd.map(str::to_owned);
            card
        };
        let cards = vec![
            with("Star", Some("*"), Some("2.00")),
            with("Three", Some("3"), None),
            with("One", Some("1"), Some("")),
            with("None", None, Some("0.50")),
            with("Two", Some("2"), Some("9.00")),
        ];
        let power = deal_order(
            cards.clone(),
            Some((CardSortKey::Power, SortDirection::Descending)),
        );
        assert_eq!(
            names(&power),
            ["Three", "Two", "One"],
            "a power the server can't read as a number is out"
        );
        let usd = deal_order(
            cards,
            Some((CardSortKey::PriceUsd, SortDirection::Ascending)),
        );
        assert_eq!(
            names(&usd),
            ["None", "Star", "Two"],
            "no price, or an empty one, is out"
        );
    }

    #[test]
    fn edhrec_and_best_match_put_unranked_last_and_best_match_ignores_direction() {
        let ranked = |name: &str, rank: Option<i32>| {
            let mut card = crate::test_support::card(name);
            card.scryfall_data.edhrec_rank = rank;
            card
        };
        let cards = vec![
            ranked("Unranked", None),
            ranked("Tenth", Some(10)),
            ranked("First", Some(1)),
        ];
        let up = deal_order(
            cards.clone(),
            Some((CardSortKey::EdhrecRank, SortDirection::Ascending)),
        );
        assert_eq!(names(&up), ["First", "Tenth", "Unranked"]);
        let down = deal_order(
            cards.clone(),
            Some((CardSortKey::EdhrecRank, SortDirection::Descending)),
        );
        assert_eq!(
            names(&down),
            ["Tenth", "First", "Unranked"],
            "unranked last both ways"
        );
        for direction in [SortDirection::Ascending, SortDirection::Descending] {
            let best = deal_order(cards.clone(), Some((CardSortKey::BestMatch, direction)));
            assert_eq!(
                names(&best),
                ["First", "Tenth", "Unranked"],
                "best first, whatever the direction"
            );
        }
    }

    #[test]
    fn a_tie_goes_by_name_then_oracle_id() {
        let cmc = |name: &str, cmc: f64| {
            let mut card = crate::test_support::card(name);
            card.scryfall_data.cmc = Some(cmc);
            card
        };
        let cards = vec![cmc("Bravo", 2.0), cmc("Alpha", 2.0), cmc("Charlie", 1.0)];
        for direction in [SortDirection::Ascending, SortDirection::Descending] {
            let sorted = deal_order(cards.clone(), Some((CardSortKey::Cmc, direction)));
            let tied: Vec<&str> = names(&sorted)
                .into_iter()
                .filter(|n| *n != "Charlie")
                .collect();
            assert_eq!(
                tied,
                ["Alpha", "Bravo"],
                "ties by name ascending either way"
            );
        }
    }

    #[test]
    fn only_a_popularity_ranking_gets_the_line() {
        assert_eq!(ranked_line(RankedBy::Popularity), Some(POPULAR_LINE));
        assert_eq!(ranked_line(RankedBy::Playstyle), None);
        assert_eq!(ranked_line(RankedBy::Unknown), None);
    }
}
