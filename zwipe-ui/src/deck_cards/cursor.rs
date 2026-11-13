//! The deck cards page's keyboard cursor (`context/plans/deck_keyboard_nav.md`): which card a
//! move lands on, given the cards as the page draws them. The browser hands in a [`Snapshot`],
//! its columns of cards top to bottom with each card's vertical center; this module decides.
//!
//! Up and down step through a column and stop at its ends. Left and right go to the nearest
//! card by vertical center in the next column that has one, and do nothing with one column.
//! Home and End go to the first and last card in reading order, columns left to right. With no
//! cursor yet, a move lands on the first card in view instead of moving.
//!
//! The theme sheet's cursor moves the same way over its rows, built with [`Snapshot::from_cells`]
//! from where each row is drawn.

use uuid::Uuid;

/// One card the cursor can stop on, as drawn. `I` names it: a card's Scryfall id on the deck
/// page, a row's place on the theme sheet.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Stop<I = Uuid> {
    /// The card's id.
    pub id: I,
    /// Its vertical center on screen, in pixels.
    pub center_y: f64,
    /// Whether any of it shows in the page's viewport.
    pub in_view: bool,
}

/// The cards as the page draws them: columns left to right, each card top to bottom. A page in
/// one column hands in one.
#[derive(Debug, Clone, PartialEq)]
pub struct Snapshot<I = Uuid> {
    /// The columns, each its cards in order.
    pub columns: Vec<Vec<Stop<I>>>,
}

impl<I> Default for Snapshot<I> {
    fn default() -> Self {
        Self {
            columns: Vec::new(),
        }
    }
}

/// One item of a grid as drawn: its stop and the left edge of its box, in pixels.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cell<I> {
    /// The item and its vertical center.
    pub stop: Stop<I>,
    /// The left edge of its box on screen, in pixels.
    pub left: f64,
}

/// How far apart two left edges may be, in pixels, and still stand in one column: subpixel
/// layout puts a column's boxes a fraction apart.
const SAME_COLUMN_PX: f64 = 2.0;

/// A cursor move.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Move {
    /// `↑` or `k`.
    Up,
    /// `↓` or `j`.
    Down,
    /// `←` or `h`.
    Left,
    /// `→` or `l`.
    Right,
    /// Home: the first card.
    First,
    /// End: the last card.
    Last,
}

/// Every key [`move_for`] answers, as the page's key listeners read them: arrows by name, letters
/// lowercase (a letter's case comes from Shift alone).
pub const MOVE_KEYS: [&str; 10] = [
    "ArrowUp",
    "ArrowDown",
    "ArrowLeft",
    "ArrowRight",
    "k",
    "j",
    "h",
    "l",
    "Home",
    "End",
];

/// The move `key` stands for.
#[must_use]
pub fn move_for(key: &str) -> Option<Move> {
    match key {
        "ArrowUp" | "k" => Some(Move::Up),
        "ArrowDown" | "j" => Some(Move::Down),
        "ArrowLeft" | "h" => Some(Move::Left),
        "ArrowRight" | "l" => Some(Move::Right),
        "Home" => Some(Move::First),
        "End" => Some(Move::Last),
        _ => None,
    }
}

impl Move {
    /// Whether the move only answers while the cursor shows: Home and End, which otherwise stay
    /// the browser's scroll.
    #[must_use]
    pub fn needs_cursor(self) -> bool {
        matches!(self, Self::First | Self::Last)
    }
}

impl<I: Copy + PartialEq> Snapshot<I> {
    /// A grid's items as columns: the items whose left edges line up form a column, the columns
    /// left to right and each top to bottom, so a move steps through the grid as drawn and a
    /// one-column list is one column.
    #[must_use]
    pub fn from_cells(cells: &[Cell<I>]) -> Self {
        let mut sorted = cells.to_vec();
        sorted.sort_by(|a, b| a.left.total_cmp(&b.left));
        let mut columns: Vec<(f64, Vec<Stop<I>>)> = Vec::new();
        for cell in sorted {
            match columns.last_mut() {
                Some((left, stops)) if (cell.left - *left).abs() <= SAME_COLUMN_PX => {
                    stops.push(cell.stop);
                }
                _ => columns.push((cell.left, vec![cell.stop])),
            }
        }
        Self {
            columns: columns
                .into_iter()
                .map(|(_, mut stops)| {
                    stops.sort_by(|a, b| a.center_y.total_cmp(&b.center_y));
                    stops
                })
                .collect(),
        }
    }

    /// The cards in reading order: the columns left to right, each top to bottom.
    fn reading_order(&self) -> impl Iterator<Item = &Stop<I>> {
        self.columns.iter().flatten()
    }

    /// Where card `id` sits: its column and its place in it.
    fn find(&self, id: I) -> Option<(usize, usize)> {
        self.columns.iter().enumerate().find_map(|(column, stops)| {
            stops
                .iter()
                .position(|stop| stop.id == id)
                .map(|at| (column, at))
        })
    }

    /// The first card in view in reading order, else the first card.
    #[must_use]
    pub fn first_in_view(&self) -> Option<I> {
        self.reading_order()
            .find(|stop| stop.in_view)
            .or_else(|| self.reading_order().next())
            .map(|stop| stop.id)
    }

    /// The card `step` from card `from` lands on, or `None` where it stops at an edge. With no
    /// `from`, or one no longer drawn, it is the card [`Snapshot::hand_on`] picks, else the first
    /// in view.
    #[must_use]
    pub fn step(&self, from: Option<I>, step: Move) -> Option<I> {
        let Some((column, at)) = from.and_then(|id| self.find(id)) else {
            return self.first_in_view();
        };
        let stops = self.columns.get(column)?;
        match step {
            Move::Up => at.checked_sub(1).and_then(|up| stops.get(up)).map(|s| s.id),
            Move::Down => stops.get(at.checked_add(1)?).map(|s| s.id),
            Move::Left | Move::Right => {
                let center = stops.get(at)?.center_y;
                self.across(column, center, step == Move::Right)
            }
            Move::First => self.reading_order().next().map(|s| s.id),
            Move::Last => self.reading_order().last().map(|s| s.id),
        }
    }

    /// The card nearest `center` by vertical center in the first column with a card, going right
    /// from `column` when `rightward`, else left.
    fn across(&self, column: usize, center: f64, rightward: bool) -> Option<I> {
        let columns: Box<dyn Iterator<Item = &Vec<Stop<I>>>> = if rightward {
            Box::new(self.columns.iter().skip(column.checked_add(1)?))
        } else {
            Box::new(self.columns.iter().take(column).rev())
        };
        let target = columns.into_iter().find(|stops| !stops.is_empty())?;
        target
            .iter()
            .min_by(|a, b| {
                let da = (a.center_y - center).abs();
                let db = (b.center_y - center).abs();
                da.total_cmp(&db)
            })
            .map(|stop| stop.id)
    }

    /// The card the cursor goes to when card `gone` leaves the page: the first card after it in
    /// `order` (the reading order before it left) that is still drawn, else the last one before
    /// it, else none.
    #[must_use]
    pub fn hand_on(&self, gone: I, order: &[I]) -> Option<I> {
        let at = order.iter().position(|id| *id == gone)?;
        let drawn = |id: &&I| self.find(**id).is_some();
        order
            .iter()
            .skip(at.saturating_add(1))
            .find(drawn)
            .or_else(|| order.iter().take(at).rev().find(drawn))
            .copied()
    }

    /// The cards' ids in reading order, for [`Snapshot::hand_on`] after a card leaves.
    #[must_use]
    pub fn order(&self) -> Vec<I> {
        self.reading_order().map(|stop| stop.id).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(n: u128) -> Uuid {
        Uuid::from_u128(n)
    }

    fn stop(n: u128, center_y: f64) -> Stop {
        Stop {
            id: id(n),
            center_y,
            in_view: true,
        }
    }

    /// Two columns: 1, 2, 3 at 10, 50, 90 on the left; 4, 5 at 20, 200 on the right.
    fn two() -> Snapshot {
        Snapshot {
            columns: vec![
                vec![stop(1, 10.0), stop(2, 50.0), stop(3, 90.0)],
                vec![stop(4, 20.0), stop(5, 200.0)],
            ],
        }
    }

    #[test]
    fn up_and_down_step_through_a_column_and_stop_at_its_ends() {
        let page = two();
        assert_eq!(page.step(Some(id(1)), Move::Down), Some(id(2)));
        assert_eq!(page.step(Some(id(2)), Move::Up), Some(id(1)));
        assert_eq!(page.step(Some(id(1)), Move::Up), None, "the top stops");
        assert_eq!(
            page.step(Some(id(3)), Move::Down),
            None,
            "the column's end stops"
        );
        assert_eq!(page.step(Some(id(5)), Move::Down), None);
    }

    #[test]
    fn left_and_right_land_on_the_nearest_card_by_center() {
        let page = two();
        assert_eq!(
            page.step(Some(id(2)), Move::Right),
            Some(id(4)),
            "50 is nearest 20"
        );
        assert_eq!(
            page.step(Some(id(5)), Move::Left),
            Some(id(3)),
            "200 is nearest 90"
        );
        assert_eq!(
            page.step(Some(id(1)), Move::Left),
            None,
            "no column to the left"
        );
        assert_eq!(
            page.step(Some(id(4)), Move::Right),
            None,
            "no column to the right"
        );
    }

    #[test]
    fn left_and_right_skip_an_empty_column_and_do_nothing_in_one_column() {
        let page = Snapshot {
            columns: vec![vec![stop(1, 10.0)], Vec::new(), vec![stop(2, 400.0)]],
        };
        assert_eq!(page.step(Some(id(1)), Move::Right), Some(id(2)));
        assert_eq!(page.step(Some(id(2)), Move::Left), Some(id(1)));
        let one = Snapshot {
            columns: vec![vec![stop(1, 10.0), stop(2, 50.0)]],
        };
        assert_eq!(one.step(Some(id(1)), Move::Right), None);
        assert_eq!(one.step(Some(id(2)), Move::Left), None);
    }

    #[test]
    fn home_and_end_go_to_the_first_and_last_card_in_reading_order() {
        let page = two();
        assert_eq!(page.step(Some(id(4)), Move::First), Some(id(1)));
        assert_eq!(page.step(Some(id(1)), Move::Last), Some(id(5)));
        assert!(Move::First.needs_cursor() && Move::Last.needs_cursor());
        assert!(!Move::Down.needs_cursor());
    }

    #[test]
    fn with_no_cursor_a_move_lands_on_the_first_card_in_view() {
        let mut page = two();
        if let Some(first) = page.columns.get_mut(0).and_then(|c| c.get_mut(0)) {
            first.in_view = false;
        }
        assert_eq!(
            page.step(None, Move::Down),
            Some(id(2)),
            "1 is scrolled away"
        );
        assert_eq!(
            page.step(Some(id(99)), Move::Up),
            Some(id(2)),
            "a card no longer drawn"
        );
        assert_eq!(Snapshot::<Uuid>::default().step(None, Move::Down), None);
    }

    #[test]
    fn a_card_that_leaves_hands_the_cursor_to_the_next_then_the_one_before() {
        let page = two();
        let before = vec![id(1), id(9), id(2), id(3)];
        assert_eq!(
            page.hand_on(id(9), &before),
            Some(id(2)),
            "the next still drawn"
        );
        let at_end = vec![id(1), id(2), id(9)];
        assert_eq!(
            page.hand_on(id(9), &at_end),
            Some(id(2)),
            "else the one before"
        );
        assert_eq!(page.hand_on(id(7), &before), None, "not in the old order");
        assert_eq!(page.order(), vec![id(1), id(2), id(3), id(4), id(5)]);
    }

    #[test]
    fn a_grid_becomes_columns_by_left_edge_each_top_to_bottom() {
        let cell = |n: usize, left: f64, center_y: f64| Cell {
            stop: Stop {
                id: n,
                center_y,
                in_view: true,
            },
            left,
        };
        let grid = Snapshot::from_cells(&[
            cell(0, 0.0, 10.0),
            cell(1, 200.0, 10.0),
            cell(2, 0.4, 60.0),
            cell(3, 199.5, 60.0),
            cell(4, 0.0, 110.0),
        ]);
        assert_eq!(grid.order(), vec![0, 2, 4, 1, 3]);
        assert_eq!(grid.step(Some(0), Move::Right), Some(1), "across a row");
        assert_eq!(grid.step(Some(3), Move::Left), Some(2));
        assert_eq!(grid.step(Some(2), Move::Down), Some(4), "down a column");
        assert_eq!(
            grid.step(Some(1), Move::Right),
            None,
            "the right edge stops"
        );
        assert_eq!(grid.step(Some(0), Move::Up), None, "the top stops");
        assert_eq!(
            grid.step(Some(3), Move::Down),
            None,
            "a short column's end stops"
        );
        let list = Snapshot::from_cells(&[cell(0, 8.0, 10.0), cell(1, 8.0, 50.0)]);
        assert_eq!(list.columns.len(), 1, "a list is one column");
        assert_eq!(list.step(Some(1), Move::Left), None);
    }

    #[test]
    fn the_keys_map_to_moves_and_letters_are_lowercase() {
        for (key, step) in [
            ("ArrowUp", Move::Up),
            ("k", Move::Up),
            ("ArrowDown", Move::Down),
            ("j", Move::Down),
            ("ArrowLeft", Move::Left),
            ("h", Move::Left),
            ("ArrowRight", Move::Right),
            ("l", Move::Right),
            ("Home", Move::First),
            ("End", Move::Last),
        ] {
            assert_eq!(move_for(key), Some(step), "{key}");
            assert!(MOVE_KEYS.contains(&key), "{key}");
        }
        assert_eq!(move_for("H"), None, "Shift+H is the shortcuts sheet");
        assert_eq!(move_for("J"), None);
    }
}
