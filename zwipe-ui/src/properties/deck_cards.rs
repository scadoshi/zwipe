//! The deck page's filter pass: a cleared filter shows the whole deck, and a filter shows
//! exactly the cards it matches, on every board.

use proptest::prelude::*;
use uuid::Uuid;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::search_card::card_filter::builder::CardQueryBuilder;
use zwipe_core::domain::card::search_card::group_cards::GroupByOption;
use zwipe_core::domain::deck::{Board, DeckEntry};

use super::strategies::{any_card, entry};
use crate::deck::form::Slot;
use crate::deck_cards::{Display, Zone, pass};

/// Any grouping the deck page offers.
fn group_by() -> impl Strategy<Value = GroupByOption> {
    prop::sample::select(vec![
        GroupByOption::CardType,
        GroupByOption::Cmc,
        GroupByOption::Color,
        GroupByOption::CardRole,
    ])
}

/// A command zone with any of its slots filled.
fn zone() -> impl Strategy<Value = Zone> {
    prop::collection::vec(prop::option::of(any_card()), 4).prop_map(|mut slots| Zone {
        signature: slots.pop().flatten(),
        background: slots.pop().flatten(),
        partner: slots.pop().flatten(),
        commander: slots.pop().flatten(),
    })
}

/// A filter by name text and type line text, each set or not.
fn filter() -> impl Strategy<Value = CardQueryBuilder> {
    (
        prop::option::of("[abc]{1,2}"),
        prop::option::of(prop::sample::select(vec!["Land", "Creature"])),
    )
        .prop_map(|(name, type_line)| {
            let mut filter = CardQueryBuilder::new();
            if let Some(name) = name {
                filter.set_name_contains(name);
            }
            if let Some(type_line) = type_line {
                filter.set_type_line_contains(type_line);
            }
            filter
        })
}

/// The printing ids, sorted, so two lists compare as multisets.
fn ids<'a>(cards: impl IntoIterator<Item = &'a Card>) -> Vec<Uuid> {
    let mut ids: Vec<Uuid> = cards.into_iter().map(|c| c.scryfall_data.id).collect();
    ids.sort_unstable();
    ids
}

/// What `display` shows on each board, as sorted printing ids, and its zone's slots.
fn shown(display: &Display) -> (Vec<Uuid>, Vec<Uuid>, Vec<Uuid>, Vec<Slot>) {
    let main = ids(display
        .groups
        .iter()
        .flat_map(|g| &g.cards)
        .chain(&display.lands));
    let maybe = ids(display.maybe.iter().map(|e| &e.card));
    let side = ids(display.side.iter().map(|e| &e.card));
    let zone = display.zone.iter().map(|(slot, _)| *slot).collect();
    (main, maybe, side, zone)
}

/// What a deck of `entries` and `zone` should show where `keep` holds.
fn expected(
    entries: &[DeckEntry],
    zone: &Zone,
    keep: impl Fn(&Card) -> bool,
) -> (Vec<Uuid>, Vec<Uuid>, Vec<Uuid>, Vec<Slot>) {
    let on = |board: Board| {
        ids(entries
            .iter()
            .filter(|e| e.deck_card.board == board && keep(&e.card))
            .map(|e| &e.card))
    };
    let zone = zone
        .cards()
        .into_iter()
        .filter(|(_, c)| keep(c))
        .map(|(slot, _)| slot)
        .collect();
    (
        on(Board::Deck),
        on(Board::Maybeboard),
        on(Board::Sideboard),
        zone,
    )
}

proptest! {
    /// Guards the cleared filter: with no filter, every card on every board and every filled
    /// zone slot shows, each once, under any grouping.
    #[test]
    fn a_cleared_filter_shows_the_whole_deck(
        entries in prop::collection::vec(entry(), 0..20),
        zone in zone(),
        group_by in group_by(),
    ) {
        let display = pass(&entries, &zone, &CardQueryBuilder::new(), group_by);
        prop_assert_eq!(shown(&display), expected(&entries, &zone, |_| true));
    }

    /// Guards the filter pass: each board and the zone show exactly the cards the filter's
    /// criteria match, each once, under any grouping.
    #[test]
    fn a_filter_shows_exactly_the_cards_it_matches(
        entries in prop::collection::vec(entry(), 0..20),
        zone in zone(),
        filter in filter(),
        group_by in group_by(),
    ) {
        let criteria = filter.build_criteria().ok();
        let keep = |card: &Card| criteria.as_ref().is_none_or(|c| c.matches(card));
        let display = pass(&entries, &zone, &filter, group_by);
        prop_assert_eq!(shown(&display), expected(&entries, &zone, keep));
    }
}
