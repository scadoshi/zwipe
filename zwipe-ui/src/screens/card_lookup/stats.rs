//! The card screen's numbers, as widgets: Decks, Swipes and, for a card that can lead a
//! Commander deck, As a commander. Each is a table whose first column names its rows and whose
//! other columns hold one number each, so a widget's column labels fit its rows.
//!
//! The numbers come from the card's stats route. Until it answers, every number reads
//! [`SOON`]; a number the screen has no figure for is never made up.

/// What a number reads while the card's stats have not arrived.
pub const SOON: &str = "Soon";

/// One cell after a row's label.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Cell {
    /// The figure isn't here yet: [`SOON`].
    Soon,
    /// The row has no figure in this column, such as a total's rate.
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

/// The widgets for a card, `leads` when it can lead a Commander deck, each number [`SOON`].
#[must_use]
pub fn widgets(leads: bool) -> Vec<Widget> {
    let soon = |label| Row {
        label,
        cells: vec![Cell::Soon],
    };
    let rate = |label| Row {
        label,
        cells: vec![Cell::Soon, Cell::Soon],
    };
    let count = |label| Row {
        label,
        cells: vec![Cell::Soon, Cell::Blank],
    };
    let decks = Widget {
        title: DECKS,
        heads: &["Stat", "Decks"],
        rows: vec![soon("In decks"), soon("MVP in decks")],
    };
    let swipes = Widget {
        title: SWIPES,
        heads: &["Swipe", "Swipes", "Rate"],
        rows: vec![
            rate("Add"),
            rate("Skip"),
            rate("Remove"),
            rate("Maybe"),
            count("Total"),
        ],
    };
    let commander = Widget {
        title: AS_COMMANDER,
        heads: &["Stat", "Count", "Rate"],
        rows: vec![count("Decks led"), rate("Pick rate")],
    };
    [decks, swipes]
        .into_iter()
        .chain(leads.then_some(commander))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn titles(leads: bool) -> Vec<&'static str> {
        widgets(leads).iter().map(|w| w.title).collect()
    }

    #[test]
    fn a_card_that_can_lead_adds_as_a_commander_after_decks_and_swipes() {
        assert_eq!(titles(false), [DECKS, SWIPES]);
        assert_eq!(titles(true), [DECKS, SWIPES, AS_COMMANDER]);
    }

    #[test]
    fn each_row_has_a_cell_for_every_column_after_its_label() {
        for widget in widgets(true) {
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

    #[test]
    fn before_the_stats_arrive_every_number_reads_soon_and_none_is_made_up() {
        for widget in widgets(true) {
            for row in &widget.rows {
                assert_eq!(
                    row.cells.first(),
                    Some(&Cell::Soon),
                    "{} / {}",
                    widget.title,
                    row.label
                );
                assert!(
                    row.cells
                        .iter()
                        .all(|c| matches!(c, Cell::Soon | Cell::Blank)),
                    "{} / {}",
                    widget.title,
                    row.label
                );
            }
        }
    }

    #[test]
    fn a_total_and_decks_led_have_no_rate() {
        let rows: Vec<Row> = widgets(true).into_iter().flat_map(|w| w.rows).collect();
        for label in ["Total", "Decks led"] {
            let row = rows.iter().find(|r| r.label == label);
            assert_eq!(
                row.map(|r| r.cells.clone()),
                Some(vec![Cell::Soon, Cell::Blank]),
                "{label}"
            );
        }
    }
}
