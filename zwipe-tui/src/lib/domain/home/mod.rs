//! The home screen's figures: the three counters, everybody's or the user's own, and the color map's rows.

use zwipe_core::http::contracts::metrics::{HttpLifetimeCounters, HttpPublicMetrics};
use zwipe_core::http::contracts::suggestions::HttpColorMap;

/// The colors in WUBRG order, by their letter.
const COLOR_LETTERS: [&str; 5] = ["W", "U", "B", "R", "G"];

/// The three counters under the logo.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct Figures {
    pub(crate) decks_created: i64,
    pub(crate) searches: i64,
    pub(crate) cards_swiped: i64,
}

impl From<&HttpPublicMetrics> for Figures {
    fn from(metrics: &HttpPublicMetrics) -> Self {
        Self {
            decks_created: metrics.decks_created,
            searches: metrics.searches,
            cards_swiped: metrics.cards_swiped,
        }
    }
}

impl From<&HttpLifetimeCounters> for Figures {
    /// The user's own counters. Every swipe direction counts as a card swiped, as in the app.
    fn from(counters: &HttpLifetimeCounters) -> Self {
        Self {
            decks_created: i64::from(counters.decks_created),
            searches: counters.searches,
            cards_swiped: [
                counters.swipes_right,
                counters.swipes_left,
                counters.swipes_up,
                counters.swipes_down,
            ]
            .into_iter()
            .fold(0, i64::saturating_add),
        }
    }
}

/// Whose counters the home screen shows.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) enum Scope {
    #[default]
    Everybody,
    OnlyMe,
}

impl Scope {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Everybody => "Everybody",
            Self::OnlyMe => "Only me",
        }
    }

    pub(crate) fn toggled(self) -> Self {
        match self {
            Self::Everybody => Self::OnlyMe,
            Self::OnlyMe => Self::Everybody,
        }
    }
}

/// One row of the color map: a color, or colorless, and how many cards across the user's decks are that color.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ColorRow {
    /// The color's place in WUBRG order, or `None` for colorless.
    pub(crate) color: Option<usize>,
    pub(crate) label: &'static str,
    pub(crate) cards: u32,
}

/// The color map's rows: the five colors in WUBRG order, then colorless.
pub(crate) fn color_rows(map: &HttpColorMap) -> Vec<ColorRow> {
    let cards_of = |letter: &str| {
        map.colors
            .iter()
            .find(|c| c.color == letter)
            .map_or(0, |c| c.cards)
    };
    COLOR_LETTERS
        .iter()
        .enumerate()
        .map(|(i, letter)| ColorRow {
            color: Some(i),
            label: letter,
            cards: cards_of(letter),
        })
        .chain([ColorRow {
            color: None,
            label: "C",
            cards: map.colorless_cards,
        }])
        .collect()
}

/// Whether the map has any cards to show, colored or colorless.
pub(crate) fn has_data(map: &HttpColorMap) -> bool {
    map.colorless_cards > 0 || map.colors.iter().any(|c| c.cards > 0)
}

/// The `n` color pairs the most decks share, as `("WU", decks)`, most first. Pairs no deck has are left out.
pub(crate) fn top_pairs(map: &HttpColorMap, n: usize) -> Vec<(String, u32)> {
    let mut pairs: Vec<(String, u32)> = map
        .pairs
        .iter()
        .filter(|p| p.decks > 0)
        .map(|p| (p.colors.concat(), p.decks))
        .collect();
    pairs.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    pairs.truncate(n);
    pairs
}

/// `1234567` as `1,234,567`.
pub(crate) fn grouped(n: i64) -> String {
    let digits = n.unsigned_abs().to_string();
    let mut out = String::with_capacity(digits.len().saturating_add(digits.len() / 3));
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len().saturating_sub(i)) % 3 == 0 {
            out.push(',');
        }
        out.push(c);
    }
    if n < 0 {
        out.insert(0, '-');
    }
    out
}

#[cfg(test)]
mod tests {
    use zwipe_core::http::contracts::suggestions::{HttpColorCards, HttpColorPair};

    use super::*;

    fn map(colors: &[(&str, u32)], colorless: u32, pairs: &[(&str, &str, u32)]) -> HttpColorMap {
        HttpColorMap {
            colors: colors
                .iter()
                .map(|&(color, cards)| HttpColorCards {
                    color: color.to_owned(),
                    cards,
                })
                .collect(),
            colorless_cards: colorless,
            pairs: pairs
                .iter()
                .map(|&(a, b, decks)| HttpColorPair {
                    colors: [a.to_owned(), b.to_owned()],
                    decks,
                })
                .collect(),
        }
    }

    #[test]
    fn my_figures_count_every_swipe_direction() {
        let counters: HttpLifetimeCounters = serde_json::from_value(serde_json::json!({
            "user_id": "00000000-0000-0000-0000-000000000000",
            "swipes_right": 5, "swipes_left": 7, "swipes_up": 1, "swipes_down": 2,
            "searches": 9, "decks_created": 3, "decks_completed": 1,
            "updated_at": "2026-01-01T00:00:00Z",
        }))
        .unwrap();
        assert_eq!(
            Figures::from(&counters),
            Figures {
                decks_created: 3,
                searches: 9,
                cards_swiped: 15
            }
        );
    }

    #[test]
    fn color_rows_fill_missing_colors_with_zero_and_end_with_colorless() {
        let rows = color_rows(&map(&[("G", 12), ("U", 4)], 3, &[]));
        let counts: Vec<(&str, u32)> = rows.iter().map(|r| (r.label, r.cards)).collect();
        assert_eq!(
            counts,
            [("W", 0), ("U", 4), ("B", 0), ("R", 0), ("G", 12), ("C", 3)]
        );
        assert_eq!(rows.last().map(|r| r.color), Some(None));
    }

    #[test]
    fn an_empty_map_has_no_data() {
        assert!(!has_data(&map(&[("W", 0)], 0, &[])));
        assert!(has_data(&map(&[], 1, &[])));
    }

    #[test]
    fn top_pairs_are_most_shared_first_and_skip_empty_pairs() {
        let pairs = top_pairs(
            &map(
                &[],
                0,
                &[("W", "U", 1), ("U", "G", 3), ("B", "R", 0), ("R", "G", 3)],
            ),
            2,
        );
        assert_eq!(pairs, [("RG".to_owned(), 3), ("UG".to_owned(), 3)]);
    }

    #[test]
    fn scope_toggles_and_names_itself() {
        assert_eq!(Scope::Everybody.toggled(), Scope::OnlyMe);
        assert_eq!(Scope::OnlyMe.toggled().label(), "Everybody");
    }

    #[test]
    fn grouped_adds_thousands_separators() {
        assert_eq!(grouped(0), "0");
        assert_eq!(grouped(999), "999");
        assert_eq!(grouped(1_000), "1,000");
        assert_eq!(grouped(1_234_567), "1,234,567");
        assert_eq!(grouped(-12_345), "-12,345");
    }
}
