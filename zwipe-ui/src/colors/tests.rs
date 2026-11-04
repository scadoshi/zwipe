use zwipe_core::domain::card::scryfall_data::colors::Colors;
use zwipe_core::domain::deck::Quantity;
use zwipe_core::http::contracts::suggestions::HttpColorCards;

use super::*;
use crate::test_support::{card, deck, deck_profile};

fn map(code: &str) -> HttpColorMap {
    HttpColorMap {
        colors: vec![HttpColorCards {
            color: code.to_owned(),
            cards: 3,
        }],
        colorless_cards: 0,
        pairs: Vec::new(),
    }
}

#[test]
fn the_bars_scale_to_the_largest_count_with_colorless_last() {
    let counts = ColorCounts {
        cards: [1, 0, 0, 0, 4],
        colorless: 2,
    };
    let rows: Vec<(Option<Color>, u32, u32)> = counts
        .bars()
        .into_iter()
        .map(|bar| (bar.color, bar.count, bar.pct))
        .collect();
    assert_eq!(
        rows,
        [
            (Some(Color::White), 1, 25),
            (Some(Color::Blue), 0, 0),
            (Some(Color::Black), 0, 0),
            (Some(Color::Red), 0, 0),
            (Some(Color::Green), 4, 100),
            (None, 2, 50),
        ]
    );
    assert!(
        ColorCounts::default().bars().iter().all(|bar| bar.pct == 0),
        "nothing to share"
    );
}

#[test]
fn a_small_count_draws_a_sliver() {
    let counts = ColorCounts {
        cards: [1, 0, 0, 0, 1000],
        colorless: 0,
    };
    let pcts: Vec<u32> = counts.bars().iter().map(|bar| bar.pct).collect();
    assert_eq!(pcts, [4, 0, 0, 0, 100, 0]);
}

#[test]
fn a_deck_s_bars_run_wubrg_then_colorless_with_zeros() {
    let colored = |name: &str, colors: &[Color]| {
        let mut card = card(name);
        card.scryfall_data.color_identity = Colors::from(colors.iter().copied());
        card
    };
    let mut deck = deck(
        deck_profile("Atraxa"),
        vec![
            colored("Lightning Bolt", &[Color::Red]),
            colored("Forest", &[Color::Green]),
            colored("Sol Ring", &[]),
            colored("Atraxa", &[Color::White, Color::Blue]),
        ],
    );
    for (entry, qty) in deck.entries.iter_mut().zip([4, 10, 1, 1]) {
        entry.deck_card.quantity = Quantity::new(qty).unwrap();
    }
    let metrics = DeckMetrics::from_entries(&deck.entries);
    let rows: Vec<(Option<Color>, u32, u32)> = ColorCounts::from(&metrics)
        .bars()
        .into_iter()
        .map(|bar| (bar.color, bar.count, bar.pct))
        .collect();
    assert_eq!(
        rows,
        [
            (Some(Color::White), 1, 10),
            (Some(Color::Blue), 1, 10),
            (Some(Color::Black), 0, 0),
            (Some(Color::Red), 4, 40),
            (Some(Color::Green), 10, 100),
            (None, 1, 10),
        ],
        "a two-color card fills both of its bars"
    );
    assert_eq!(
        ColorCounts::from(&DeckMetrics::from_entries(&[])),
        ColorCounts::default()
    );
}

#[test]
fn the_counts_read_each_color_by_name() {
    let counts = ColorCounts::from(&HttpColorMap {
        colors: vec![
            HttpColorCards {
                color: "G".to_owned(),
                cards: 4,
            },
            HttpColorCards {
                color: "W".to_owned(),
                cards: 1,
            },
        ],
        colorless_cards: 2,
        pairs: Vec::new(),
    });
    assert_eq!(
        counts,
        ColorCounts {
            cards: [1, 0, 0, 0, 4],
            colorless: 2,
        }
    );
}

#[test]
fn the_counts_have_data_with_any_colored_or_colorless_card() {
    assert!(!ColorCounts::default().has_cards());
    assert!(
        ColorCounts {
            colorless: 1,
            ..ColorCounts::default()
        }
        .has_cards()
    );
    assert!(
        ColorCounts {
            cards: [0, 0, 0, 0, 2],
            ..ColorCounts::default()
        }
        .has_cards()
    );
}

#[test]
fn only_wubrg_codes_count_as_data() {
    assert!(ColorCounts::from(&map("G")).has_cards());
    assert!(
        !ColorCounts::from(&map("C")).has_cards(),
        "a code outside WUBRG is no data"
    );
}
