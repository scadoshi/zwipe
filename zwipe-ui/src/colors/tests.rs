use zwipe_core::http::contracts::suggestions::HttpColorCards;

use super::*;

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
    let rows: Vec<(Option<Color>, u32, u32)> = bars(&counts)
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
        bars(&ColorCounts::default()).iter().all(|bar| bar.pct == 0),
        "nothing to share"
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
    assert!(!has_data(&ColorCounts::default()));
    assert!(has_data(&ColorCounts {
        colorless: 1,
        ..ColorCounts::default()
    }));
    assert!(has_data(&ColorCounts {
        cards: [0, 0, 0, 0, 2],
        ..ColorCounts::default()
    }));
}

#[test]
fn only_wubrg_codes_count_as_data() {
    assert!(has_data(&ColorCounts::from(&map("G"))));
    assert!(
        !has_data(&ColorCounts::from(&map("C"))),
        "a code outside WUBRG is no data"
    );
}
