//! The in-memory card filter keeps a stable sub-list, however often it runs.

use proptest::prelude::*;

use super::strategies::{card, colors};
use crate::domain::card::Card;
use crate::domain::card::scryfall_data::colors::Colors;
use crate::domain::card::search_card::card_filter::CardCriteria;
use crate::domain::card::search_card::card_filter::builder::CardQueryBuilder;
use crate::domain::card::search_card::cards::Cards;

/// A filter over the fields the generated cards vary: name text, a mana value range, a color
/// identity and type line text, each set or not.
fn filter() -> impl Strategy<Value = CardQueryBuilder> {
    (
        prop::option::of("[abc]{1,2}"),
        prop::option::of((0u8..=8, 0u8..=8)),
        prop::option::of(colors()),
        prop::option::of(prop::sample::select(vec![
            "Creature",
            "Land",
            "Instant",
            "Legendary",
        ])),
    )
        .prop_map(|(name, cmc, within, type_line)| {
            let mut filter = CardQueryBuilder::new();
            if let Some(name) = name {
                filter.set_name_contains(name);
            }
            if let Some((a, b)) = cmc {
                filter.set_cmc_range((f64::from(a.min(b)), f64::from(a.max(b))));
            }
            if let Some(within) = within {
                filter.set_color_identity_within(Colors::from(within));
            }
            if let Some(type_line) = type_line {
                filter.set_type_line_contains(type_line);
            }
            filter
        })
}

proptest! {
    /// Guards filtering a pile: filtering twice gives what filtering once gave, and that is
    /// exactly the cards that match, in their order.
    #[test]
    fn filtering_twice_is_filtering_once(
        cards in prop::collection::vec(card(), 0..24),
        filter in filter(),
    ) {
        let criteria = filter.build_criteria().unwrap_or_default();
        let once = Cards::from(cards.clone()).matching(&criteria);
        let twice = once.clone().matching(&criteria);
        prop_assert_eq!(&twice, &once);
        let matching: Vec<Card> = cards.into_iter().filter(|c| criteria.matches(c)).collect();
        prop_assert_eq!(Vec::<Card>::from(once), matching);
    }

    /// Guards a cleared filter: no criteria keep every card, in order.
    #[test]
    fn no_criteria_keep_every_card(cards in prop::collection::vec(card(), 0..24)) {
        let kept: Vec<Card> = Cards::from(cards.clone()).matching(&CardCriteria::default()).into();
        prop_assert_eq!(kept, cards);
    }
}
