//! A deck's total and the copy gate: a card stops at 99 and a deck at its card limit.

use proptest::prelude::*;
use zwipe_core::domain::deck::{DeckEntry, Quantity};

use super::strategies::entry;
use crate::deck::copies::{can_add_copy, card_limit, deck_total};

proptest! {
    /// Guards the count every limit reads: the total is every copy on every board plus each
    /// filled command zone slot, and one more copy raises it by exactly one.
    #[test]
    fn the_total_is_every_copy_plus_the_zone_and_a_copy_adds_one(
        entries in prop::collection::vec(entry(), 0..20),
        zone in 0usize..=4,
        pick in any::<prop::sample::Index>(),
    ) {
        let copies: i64 = entries.iter().map(|e| i64::from(*e.deck_card.quantity)).sum();
        let zone_count = i64::try_from(zone).unwrap();
        let total = deck_total(&entries, zone);
        prop_assert_eq!(total, copies.checked_add(zone_count).unwrap());

        if !entries.is_empty() {
            let mut more = entries.clone();
            let picked = &mut more[pick.index(entries.len())].deck_card.quantity;
            if let Ok(next) = Quantity::new(picked.checked_add(1).unwrap()) {
                *picked = next;
                prop_assert_eq!(deck_total(&more, zone), total.checked_add(1).unwrap());
            }
        }
    }

    /// Guards the copy gate: adds that ask `can_add_copy` first never take a card past 99 or a
    /// deck past its card limit, verified or not, and a deck already past it never grows.
    #[test]
    fn gated_adds_never_pass_99_copies_or_the_card_limit(
        mut entries in prop::collection::vec(entry(), 1..20),
        zone in 0usize..=4,
        verified in any::<bool>(),
        picks in prop::collection::vec(any::<prop::sample::Index>(), 0..300),
    ) {
        let limit = card_limit(verified);
        let start = deck_total(&entries, zone);
        for pick in picks {
            let at = pick.index(entries.len());
            let quantity = *entries[at].deck_card.quantity;
            if can_add_copy(quantity, deck_total(&entries, zone), limit) {
                entries[at].deck_card.quantity = Quantity::new(quantity.checked_add(1).unwrap()).unwrap();
            }
        }
        let end = deck_total(&entries, zone);
        prop_assert!(entries.iter().all(|e: &DeckEntry| *e.deck_card.quantity <= Quantity::MAX));
        if start <= limit {
            prop_assert!(end <= limit, "{start} grew to {end} past {limit}");
        } else {
            prop_assert_eq!(end, start);
        }
    }
}
