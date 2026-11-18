//! Resetting a filter: it comes back blank, and a second reset has nothing to do.

use proptest::prelude::*;
use zwipe_core::domain::card::search_card::card_filter::CardSortKey;
use zwipe_core::domain::card::search_card::card_filter::builder::CardQueryBuilder;

use crate::filter::{AlreadyDefault, reset_maybeboard};

/// A filter with any of a name, a mana value range, a type line, a sort and synergy set.
fn filter() -> impl Strategy<Value = CardQueryBuilder> {
    (
        prop::option::of("[abc]{1,3}"),
        prop::option::of((0u8..=8, 0u8..=8)),
        prop::option::of(prop::sample::select(vec!["Creature", "Land"])),
        prop::option::of(prop::sample::select(vec![
            CardSortKey::Name,
            CardSortKey::Cmc,
            CardSortKey::PriceUsd,
        ])),
        any::<bool>(),
    )
        .prop_map(|(name, cmc, type_line, sort, synergy)| {
            let mut filter = CardQueryBuilder::new();
            if let Some(name) = name {
                filter.set_name_contains(name);
            }
            if let Some((a, b)) = cmc {
                filter.set_cmc_range((f64::from(a.min(b)), f64::from(a.max(b))));
            }
            if let Some(type_line) = type_line {
                filter.set_type_line_contains(type_line);
            }
            if let Some(sort) = sort {
                filter.set_sort(sort);
            }
            filter.set_synergy(synergy);
            filter
        })
}

proptest! {
    /// Guards Clear: any filter clears to the default one.
    #[test]
    fn clear_returns_any_filter_to_the_default(mut filter in filter()) {
        filter.clear();
        prop_assert_eq!(filter, CardQueryBuilder::default());
    }

    /// Guards the maybeboard's Reset: a filter with criteria or a sort resets to blank keeping
    /// only its synergy, a blank one answers that it already is, and a second Reset changes
    /// nothing.
    #[test]
    fn the_maybeboard_reset_blanks_once_then_has_nothing_to_do(mut filter in filter()) {
        let synergy = filter.synergy();
        let had_something = !filter.is_empty() || filter.sort().is_some();
        let first = reset_maybeboard(&mut filter);
        prop_assert_eq!(first.is_ok(), had_something);

        let mut blank = CardQueryBuilder::default();
        blank.set_synergy(synergy);
        if had_something {
            prop_assert_eq!(&filter, &blank);
        }
        let before = filter.clone();
        prop_assert_eq!(reset_maybeboard(&mut filter), Err(AlreadyDefault));
        prop_assert_eq!(filter, before);
    }
}
