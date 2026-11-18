//! Generators shared by the property tests.

use proptest::prelude::*;

use crate::domain::card::Card;
use crate::domain::card::scryfall_data::colors::{Color, Colors};
use crate::test_utils::make_card;

/// Any list of colors, repeats and any order included.
pub(super) fn colors() -> impl Strategy<Value = Vec<Color>> {
    prop::collection::vec(prop::sample::select(Color::all().to_vec()), 0..12)
}

/// A type line from the shapes the filters read: creatures, spells, lands and a legend.
const TYPE_LINES: [&str; 6] = [
    "Creature — Elf Druid",
    "Instant",
    "Sorcery",
    "Basic Land — Forest",
    "Artifact Creature — Golem",
    "Legendary Planeswalker — Nissa",
];

/// A card with a short name, a mana value, a type line and a color identity, the fields the
/// in-memory filters read.
pub(super) fn card() -> impl Strategy<Value = Card> {
    (
        "[abc]{1,4}",
        0u8..=8,
        prop::sample::select(TYPE_LINES.to_vec()),
        colors(),
    )
        .prop_map(|(name, cmc, type_line, identity)| {
            let mut card = make_card(&name);
            let data = &mut card.scryfall_data;
            data.cmc = Some(f64::from(cmc));
            data.type_line = Some(type_line.to_owned());
            data.color_identity = Colors::from(identity);
            card
        })
}
