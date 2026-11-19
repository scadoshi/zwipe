//! The card screen's numbers, as widgets: Decks, Swipes and, for a card that can lead a
//! Commander deck, As a commander. Each is a table whose first column names its rows and whose
//! other columns hold one figure each, so a widget's column labels fit its rows.
//!
//! The figures come from the card's stats route ([`HttpCardStats`]), which sends counts; the
//! rates are taken here, as whole percents rounded down. Add, Skip and Maybe are shares of the
//! times the card was dealt, which Total shows; Remove is a share of the adds; a pick is a share
//! of the times Swipe select dealt the card. A share of nothing has no figure, and nothing is
//! ever made up: before the stats arrive every figure is [`Cell::Loading`].

use orangui_core::number;
use zwipe_core::http::contracts::card::{HttpCardStats, HttpCardSwipes, HttpCommanderPicks};

/// One cell after a row's label.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Cell {
    /// The stats haven't arrived.
    Loading,
    /// A count or a rate, ready to draw.
    Figure(String),
    /// No figure in this column, such as a total's rate or a share of nothing.
    Blank,
}

/// One row of a widget: its label, then a cell per column after the first.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    /// The row's name.
    pub label: &'static str,
    /// The row's figures, one per column after the label.
    pub cells: Vec<Cell>,
}

/// One stats widget: a fold titled `title` holding a table of `heads` over `rows`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Widget {
    /// The fold's title.
    pub title: &'static str,
    /// The column labels, the first naming the rows.
    pub heads: &'static [&'static str],
    /// The table's rows, in the order they show.
    pub rows: Vec<Row>,
}

/// The decks a card is in.
pub const DECKS: &str = "Decks";
/// How the card is swiped.
pub const SWIPES: &str = "Swipes";
/// The card as a commander.
pub const AS_COMMANDER: &str = "As a commander";

/// The widgets for a card, `leads` when it can lead a Commander deck, over its `stats`, or every
/// figure [`Cell::Loading`] while they haven't arrived.
#[must_use]
pub fn widgets(stats: Option<&HttpCardStats>, leads: bool) -> Vec<Widget> {
    let decks = Widget {
        title: DECKS,
        heads: &["Stat", "Decks"],
        rows: vec![
            row("In decks", vec![count(stats, |s| s.in_decks)]),
            row("MVP in decks", vec![count(stats, |s| s.mvp_decks)]),
        ],
    };
    let swipes = stats.map(|s| s.swipes.unwrap_or_default());
    let swipe = |label, kind: fn(&HttpCardSwipes) -> (u64, u64)| {
        row(label, share(swipes.as_ref().map(kind)))
    };
    let swipes_widget = Widget {
        title: SWIPES,
        heads: &["Swipe", "Swipes", "Rate"],
        rows: vec![
            swipe("Add", |s| (s.added, s.shown)),
            swipe("Skip", |s| (s.skipped, s.shown)),
            swipe("Remove", |s| (s.removed, s.added)),
            swipe("Maybe", |s| (s.maybed, s.shown)),
            row("Total", vec![loaded(swipes.map(|s| s.shown)), Cell::Blank]),
        ],
    };
    let picks = stats.map(|s| s.commander_picks.unwrap_or_default());
    let commander = Widget {
        title: AS_COMMANDER,
        heads: &["Stat", "Count", "Rate"],
        rows: vec![
            row(
                "Decks led",
                vec![count(stats, |s| s.decks_led), Cell::Blank],
            ),
            row(
                "Pick rate",
                share(picks.map(|p: HttpCommanderPicks| (p.selected, p.shown))),
            ),
        ],
    };
    [decks, swipes_widget]
        .into_iter()
        .chain(leads.then_some(commander))
        .collect()
}

/// A row of `cells` under `label`.
fn row(label: &'static str, cells: Vec<Cell>) -> Row {
    Row { label, cells }
}

/// The cell for the count `field` reads from `stats`: loading while they haven't arrived, blank
/// when they don't carry it.
fn count(stats: Option<&HttpCardStats>, field: fn(&HttpCardStats) -> Option<u64>) -> Cell {
    let Some(stats) = stats else {
        return Cell::Loading;
    };
    field(stats).map_or(Cell::Blank, |n| Cell::Figure(number::with_separators(n)))
}

/// A count that is there once the stats are: loading until then.
fn loaded(n: Option<u64>) -> Cell {
    n.map_or(Cell::Loading, |n| Cell::Figure(number::with_separators(n)))
}

/// A count and its share of `of`: loading while the stats haven't arrived, and no rate when
/// there is nothing to take a share of.
fn share(parts: Option<(u64, u64)>) -> Vec<Cell> {
    let Some((n, of)) = parts else {
        return vec![Cell::Loading, Cell::Loading];
    };
    let rate = percent(n, of).map_or(Cell::Blank, |pct| Cell::Figure(format!("{pct}%")));
    vec![Cell::Figure(number::with_separators(n)), rate]
}

/// `n` as a whole percent of `of`, rounded down and at most 100; `None` when `of` is zero.
fn percent(n: u64, of: u64) -> Option<u64> {
    n.min(of).saturating_mul(100).checked_div(of)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn figure(text: &str) -> Cell {
        Cell::Figure(text.to_owned())
    }

    /// The cells of the row labeled `label`, across every widget.
    fn cells(widgets: &[Widget], label: &str) -> Vec<Cell> {
        let Some(row) = widgets
            .iter()
            .flat_map(|w| &w.rows)
            .find(|r| r.label == label)
        else {
            panic!("no row {label}");
        };
        row.cells.clone()
    }

    /// Thinker's example answer.
    fn example() -> HttpCardStats {
        HttpCardStats {
            in_decks: Some(2),
            mvp_decks: Some(1),
            swipes: Some(HttpCardSwipes {
                shown: 50,
                added: 30,
                skipped: 15,
                maybed: 5,
                removed: 3,
            }),
            decks_led: Some(0),
            commander_picks: None,
        }
    }

    #[test]
    fn a_card_that_can_lead_adds_as_a_commander_after_decks_and_swipes() {
        let titles = |leads| -> Vec<&str> {
            widgets(Some(&example()), leads)
                .iter()
                .map(|w| w.title)
                .collect()
        };
        assert_eq!(titles(false), [DECKS, SWIPES]);
        assert_eq!(titles(true), [DECKS, SWIPES, AS_COMMANDER]);
    }

    #[test]
    fn each_row_has_a_cell_for_every_column_after_its_label() {
        for stats in [None, Some(example()), Some(HttpCardStats::default())] {
            for widget in widgets(stats.as_ref(), true) {
                for row in &widget.rows {
                    assert_eq!(
                        row.cells.len().saturating_add(1),
                        widget.heads.len(),
                        "{} / {}",
                        widget.title,
                        row.label
                    );
                }
            }
        }
    }

    #[test]
    fn before_the_stats_arrive_every_figure_is_loading_and_none_is_made_up() {
        for widget in widgets(None, true) {
            for row in &widget.rows {
                assert_eq!(row.cells.first(), Some(&Cell::Loading), "{}", row.label);
                assert!(
                    row.cells
                        .iter()
                        .all(|c| matches!(c, Cell::Loading | Cell::Blank)),
                    "{}",
                    row.label
                );
            }
        }
    }

    #[test]
    fn swipe_rates_are_shares_of_the_deals_and_remove_a_share_of_the_adds() {
        let w = widgets(Some(&example()), true);
        assert_eq!(cells(&w, "In decks"), [figure("2")]);
        assert_eq!(cells(&w, "MVP in decks"), [figure("1")]);
        assert_eq!(cells(&w, "Add"), [figure("30"), figure("60%")]);
        assert_eq!(cells(&w, "Skip"), [figure("15"), figure("30%")]);
        assert_eq!(cells(&w, "Remove"), [figure("3"), figure("10%")]);
        assert_eq!(cells(&w, "Maybe"), [figure("5"), figure("10%")]);
        assert_eq!(cells(&w, "Total"), [figure("50"), Cell::Blank]);
        assert_eq!(cells(&w, "Decks led"), [figure("0"), Cell::Blank]);
    }

    #[test]
    fn a_card_nobody_has_swiped_or_picked_shows_zero_and_no_rate() {
        let w = widgets(Some(&example()), true);
        assert_eq!(cells(&w, "Pick rate"), [figure("0"), Cell::Blank]);
        let untouched = HttpCardStats {
            in_decks: Some(0),
            mvp_decks: Some(0),
            decks_led: Some(0),
            ..HttpCardStats::default()
        };
        let w = widgets(Some(&untouched), true);
        assert_eq!(cells(&w, "Add"), [figure("0"), Cell::Blank]);
        assert_eq!(cells(&w, "Total"), [figure("0"), Cell::Blank]);
        assert_eq!(cells(&w, "In decks"), [figure("0")]);
    }

    #[test]
    fn rates_round_down_cap_at_a_hundred_and_counts_take_separators() {
        let stats = HttpCardStats {
            swipes: Some(HttpCardSwipes {
                shown: 1_722,
                added: 1_240,
                skipped: 310,
                maybed: 103,
                removed: 1_300,
            }),
            commander_picks: Some(HttpCommanderPicks {
                shown: 3,
                selected: 1,
            }),
            ..example()
        };
        let w = widgets(Some(&stats), true);
        assert_eq!(cells(&w, "Add"), [figure("1,240"), figure("72%")]);
        assert_eq!(cells(&w, "Total"), [figure("1,722"), Cell::Blank]);
        assert_eq!(
            cells(&w, "Remove"),
            [figure("1,300"), figure("100%")],
            "more removals than adds caps the rate"
        );
        assert_eq!(cells(&w, "Pick rate"), [figure("1"), figure("33%")]);
    }

    #[test]
    fn a_count_an_older_server_leaves_out_is_blank_not_zero() {
        let w = widgets(Some(&HttpCardStats::default()), true);
        assert_eq!(cells(&w, "In decks"), [Cell::Blank]);
        assert_eq!(cells(&w, "Decks led"), [Cell::Blank, Cell::Blank]);
    }
}
