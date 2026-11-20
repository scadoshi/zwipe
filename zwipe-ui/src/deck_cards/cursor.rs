//! The deck cards page's keyboard cursor: which card a
//! move lands on, given the cards as the page draws them, in columns. orangui-core's
//! [`grid_cursor`](orangui_core::grid_cursor) decides; here a card is named by its Scryfall id.
//! A screen of card lists side by side walks them as [`Walk::Lists`], and a grid of cards (the
//! card search's results) as [`Walk::Grid`].

pub use orangui_core::grid_cursor::{Cell, MOVE_KEYS, Move, move_for};
use uuid::Uuid;

/// One card the cursor can stop on, as drawn, named by its Scryfall id.
pub type Stop<I = Uuid> = orangui_core::grid_cursor::Stop<I>;

/// The cards as the page draws them: columns left to right, each card top to bottom.
pub type Snapshot<I = Uuid> = orangui_core::grid_cursor::Snapshot<I>;

/// How a screen's cursor walks its columns.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Walk {
    /// The deck page's columns: up and down stop at a column's ends, and left and right go to the
    /// nearest card by height in the next column ([`Snapshot::step`]).
    Columns,
    /// Lists side by side (Playstyle's Suggested commanders and Staples): up and down walk every
    /// card in reading order, from the end of one list on to the top of the next, and left and
    /// right go to the card at the same place in the next list with cards, else its last card.
    /// One list walks as one column.
    Lists,
    /// A grid of cards laid out in rows (the card search's results), each column one of its
    /// columns: left and right go along the row and stop at its ends, up and down go to the card
    /// above or below in the same column, and down from a column with no card in the next row
    /// goes to that row's last card. Home and End go to the first and last card in reading order.
    Grid,
}

/// The card `step` from card `from` lands on as `walk` moves, or `None` where it stops at an
/// edge. With no `from`, or one no longer drawn, it is the first card in view.
#[must_use]
pub fn step<I: Copy + PartialEq>(
    snapshot: &Snapshot<I>,
    from: Option<I>,
    step: Move,
    walk: Walk,
) -> Option<I> {
    let place = from.and_then(|id| {
        snapshot
            .columns
            .iter()
            .enumerate()
            .find_map(|(column, stops)| {
                stops
                    .iter()
                    .position(|stop| stop.id == id)
                    .map(|at| (column, at))
            })
    });
    if let (Walk::Grid, Some(place)) = (walk, place) {
        return grid_step(snapshot, place, step);
    }
    let (Walk::Lists, Some((column, at)), Some(id)) = (walk, place, from) else {
        return snapshot.step(from, step);
    };
    match step {
        Move::Up | Move::Down => {
            let order = snapshot.order();
            let here = order.iter().position(|other| *other == id)?;
            let there = if step == Move::Down {
                here.checked_add(1)?
            } else {
                here.checked_sub(1)?
            };
            order.get(there).copied()
        }
        Move::Left | Move::Right => {
            let mut columns: Box<dyn Iterator<Item = &Vec<Stop<I>>>> = if step == Move::Right {
                Box::new(snapshot.columns.iter().skip(column.checked_add(1)?))
            } else {
                Box::new(snapshot.columns.iter().take(column).rev())
            };
            let list = columns.find(|stops| !stops.is_empty())?;
            list.get(at).or_else(|| list.last()).map(|stop| stop.id)
        }
        Move::First | Move::Last => snapshot.step(from, step),
    }
}

/// Where `step` lands in a grid whose columns are `snapshot`'s, from the card at index `at` of
/// column `column` ([`Walk::Grid`]). A card's index in its column is its row.
fn grid_step<I: Copy>(
    snapshot: &Snapshot<I>,
    (column, at): (usize, usize),
    step: Move,
) -> Option<I> {
    let card = |column: usize, row: usize| {
        snapshot
            .columns
            .get(column)
            .and_then(|stops| stops.get(row))
            .map(|stop| stop.id)
    };
    match step {
        Move::Left => card(column.checked_sub(1)?, at),
        Move::Right => card(column.checked_add(1)?, at),
        Move::Up => card(column, at.checked_sub(1)?),
        Move::Down => {
            let below = at.checked_add(1)?;
            card(column, below).or_else(|| (0..column).rev().find_map(|left| card(left, below)))
        }
        Move::First => card(0, 0),
        Move::Last => grid_reading_order(snapshot).last().copied(),
    }
}

/// A grid's cards in reading order, row by row and each row left to right.
fn grid_reading_order<I: Copy>(snapshot: &Snapshot<I>) -> Vec<I> {
    let rows = snapshot.columns.iter().map(Vec::len).max().unwrap_or(0);
    (0..rows)
        .flat_map(|row| {
            snapshot
                .columns
                .iter()
                .filter_map(move |stops| stops.get(row).map(|stop| stop.id))
        })
        .collect()
}

/// One card of the row above the columns (the deck page's featured cards), at its horizontal
/// center.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HeadStop<I = Uuid> {
    /// The stop's name.
    pub id: I,
    /// Its horizontal center, in the page's pixels.
    pub center_x: f64,
}

/// The cards drawn above the columns, left to right, with each column's horizontal center, so a
/// move between the row and the columns lands on the nearest card across. A page with no such
/// row has none ([`Head::default`]) and moves as [`step`] does.
#[derive(Debug, Clone, PartialEq)]
pub struct Head<I = Uuid> {
    /// The row's cards, left to right.
    pub stops: Vec<HeadStop<I>>,
    /// Each column's horizontal center, in the snapshot's column order.
    pub column_x: Vec<f64>,
}

impl<I> Default for Head<I> {
    fn default() -> Self {
        Self {
            stops: Vec::new(),
            column_x: Vec::new(),
        }
    }
}

/// [`step`] on a page with a row of cards above its columns: up from a column's top card goes to
/// the row's card nearest above it, left and right walk the row and stop at its ends, down from
/// the row goes to the top card of the column nearest below, and nothing is above the row. Home
/// and End go to the first and last card in the columns from anywhere.
#[must_use]
pub fn step_with_head<I: Copy + PartialEq>(
    snapshot: &Snapshot<I>,
    head: &Head<I>,
    from: Option<I>,
    step: Move,
    walk: Walk,
) -> Option<I> {
    let in_head = from.and_then(|id| head.stops.iter().position(|stop| stop.id == id));
    if let Some(at) = in_head {
        return match step {
            Move::Up => None,
            Move::Left => at
                .checked_sub(1)
                .and_then(|left| head.stops.get(left))
                .map(|stop| stop.id),
            Move::Right => head.stops.get(at.checked_add(1)?).map(|stop| stop.id),
            Move::Down => {
                let center = head.stops.get(at)?.center_x;
                snapshot
                    .columns
                    .iter()
                    .zip(&head.column_x)
                    .filter_map(|(stops, x)| stops.first().map(|top| (top.id, (x - center).abs())))
                    .min_by(|a, b| a.1.total_cmp(&b.1))
                    .map(|(id, _)| id)
            }
            Move::First => snapshot.order().first().copied(),
            Move::Last => snapshot.order().last().copied(),
        };
    }
    let landed = self::step(snapshot, from, step, walk);
    if landed.is_some() || step != Move::Up {
        return landed;
    }
    let column = from.and_then(|id| {
        snapshot
            .columns
            .iter()
            .position(|stops| stops.iter().any(|stop| stop.id == id))
    })?;
    let center = *head.column_x.get(column)?;
    head.stops
        .iter()
        .min_by(|a, b| {
            (a.center_x - center)
                .abs()
                .total_cmp(&(b.center_x - center).abs())
        })
        .map(|stop| stop.id)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stop(n: u128, center_y: f64) -> Stop<u128> {
        Stop {
            id: n,
            center_y,
            in_view: true,
        }
    }

    /// Commanders 1 to 4 on the left, staples 11 and 12 on the right, the staples' rows taller
    /// and lower, so a card's place and its height disagree.
    fn lists() -> Snapshot<u128> {
        Snapshot {
            columns: vec![
                vec![
                    stop(1, 100.0),
                    stop(2, 140.0),
                    stop(3, 180.0),
                    stop(4, 220.0),
                ],
                vec![stop(11, 150.0), stop(12, 230.0)],
            ],
        }
    }

    #[test]
    fn left_and_right_go_to_the_same_place_in_the_other_list() {
        let page = lists();
        assert_eq!(
            step(&page, Some(1), Move::Right, Walk::Lists),
            Some(11),
            "the first commander to the first staple"
        );
        assert_eq!(
            step(&page, Some(2), Move::Right, Walk::Lists),
            Some(12),
            "the second commander to the second staple, though the first is nearer by height"
        );
        assert_eq!(
            step(&page, Some(12), Move::Left, Walk::Lists),
            Some(2),
            "the second staple to the second commander"
        );
        assert_eq!(step(&page, Some(11), Move::Left, Walk::Lists), Some(1));
    }

    #[test]
    fn left_and_right_clamp_to_the_other_list_s_last_card() {
        let page = lists();
        assert_eq!(
            step(&page, Some(4), Move::Right, Walk::Lists),
            Some(12),
            "the fourth commander to the last of two staples"
        );
        assert_eq!(
            step(&page, Some(4), Move::Left, Walk::Lists),
            None,
            "no list to the left"
        );
        assert_eq!(
            step(&page, Some(12), Move::Right, Walk::Lists),
            None,
            "no list to the right"
        );
    }

    #[test]
    fn left_and_right_skip_a_list_with_no_cards_and_do_nothing_in_one_list() {
        let page = Snapshot {
            columns: vec![vec![stop(1, 10.0), stop(2, 50.0)], Vec::new()],
        };
        assert_eq!(step(&page, Some(2), Move::Right, Walk::Lists), None);
        let one = Snapshot {
            columns: vec![vec![stop(1, 10.0), stop(2, 50.0)]],
        };
        assert_eq!(step(&one, Some(1), Move::Right, Walk::Lists), None);
        assert_eq!(step(&one, Some(2), Move::Left, Walk::Lists), None);
    }

    #[test]
    fn up_and_down_walk_on_from_one_list_to_the_next() {
        let page = lists();
        assert_eq!(
            step(&page, Some(4), Move::Down, Walk::Lists),
            Some(11),
            "the last commander on to the first staple"
        );
        assert_eq!(
            step(&page, Some(11), Move::Up, Walk::Lists),
            Some(4),
            "and back"
        );
        assert_eq!(step(&page, Some(2), Move::Down, Walk::Lists), Some(3));
        assert_eq!(
            step(&page, Some(12), Move::Down, Walk::Lists),
            None,
            "the last staple stops"
        );
        assert_eq!(
            step(&page, Some(1), Move::Up, Walk::Lists),
            None,
            "the first commander stops"
        );
    }

    #[test]
    fn home_end_and_no_cursor_walk_as_the_deck_page_does() {
        let page = lists();
        assert_eq!(step(&page, Some(12), Move::First, Walk::Lists), Some(1));
        assert_eq!(step(&page, Some(1), Move::Last, Walk::Lists), Some(12));
        assert_eq!(
            step(&page, None, Move::Right, Walk::Lists),
            Some(1),
            "the first card in view"
        );
        assert_eq!(
            step(&page, Some(99), Move::Down, Walk::Lists),
            Some(1),
            "a card no longer drawn"
        );
    }

    #[test]
    fn the_deck_page_s_columns_walk_as_before() {
        let page = lists();
        for (from, move_to) in [
            (Some(4), Move::Down),
            (Some(11), Move::Up),
            (Some(1), Move::Right),
            (Some(12), Move::Left),
            (Some(4), Move::Right),
            (None, Move::Down),
        ] {
            assert_eq!(
                step(&page, from, move_to, Walk::Columns),
                page.step(from, move_to),
                "{from:?} {move_to:?}"
            );
        }
    }

    /// A grid of four columns and three rows, the last row holding two cards: 1 2 3 4, 5 6 7 8,
    /// 9 10. Each column is a column of the grid, top to bottom.
    fn grid() -> Snapshot<u128> {
        Snapshot {
            columns: vec![
                vec![stop(1, 100.0), stop(5, 300.0), stop(9, 500.0)],
                vec![stop(2, 100.0), stop(6, 300.0), stop(10, 500.0)],
                vec![stop(3, 100.0), stop(7, 300.0)],
                vec![stop(4, 100.0), stop(8, 300.0)],
            ],
        }
    }

    #[test]
    fn left_and_right_go_along_the_row_and_stop_at_its_ends() {
        let page = grid();
        assert_eq!(step(&page, Some(2), Move::Right, Walk::Grid), Some(3));
        assert_eq!(step(&page, Some(6), Move::Left, Walk::Grid), Some(5));
        assert_eq!(
            step(&page, Some(4), Move::Right, Walk::Grid),
            None,
            "the row's end"
        );
        assert_eq!(
            step(&page, Some(5), Move::Left, Walk::Grid),
            None,
            "the row's start"
        );
        assert_eq!(
            step(&page, Some(10), Move::Right, Walk::Grid),
            None,
            "the short last row ends at its last card"
        );
    }

    #[test]
    fn up_and_down_keep_the_column() {
        let page = grid();
        assert_eq!(step(&page, Some(2), Move::Down, Walk::Grid), Some(6));
        assert_eq!(step(&page, Some(6), Move::Down, Walk::Grid), Some(10));
        assert_eq!(step(&page, Some(7), Move::Up, Walk::Grid), Some(3));
        assert_eq!(
            step(&page, Some(3), Move::Up, Walk::Grid),
            None,
            "the top row"
        );
        assert_eq!(
            step(&page, Some(10), Move::Down, Walk::Grid),
            None,
            "the last row"
        );
    }

    #[test]
    fn down_into_a_short_last_row_lands_on_its_last_card() {
        let page = grid();
        assert_eq!(step(&page, Some(8), Move::Down, Walk::Grid), Some(10));
        assert_eq!(step(&page, Some(7), Move::Down, Walk::Grid), Some(10));
    }

    #[test]
    fn home_and_end_go_to_the_first_and_last_card_in_reading_order() {
        let page = grid();
        assert_eq!(step(&page, Some(7), Move::First, Walk::Grid), Some(1));
        assert_eq!(
            step(&page, Some(1), Move::Last, Walk::Grid),
            Some(10),
            "the last row's last card, not the last column's"
        );
        assert_eq!(
            step(&page, None, Move::Right, Walk::Grid),
            Some(1),
            "with no cursor, the first card in view"
        );
    }

    /// Two deck columns, their centers at x 100 and 400, under featured cards 91 (x 80), 92
    /// (x 250) and 93 (x 420).
    fn headed() -> (Snapshot<u128>, Head<u128>) {
        let page = Snapshot {
            columns: vec![
                vec![stop(1, 300.0), stop(2, 340.0)],
                vec![stop(11, 300.0), stop(12, 360.0)],
            ],
        };
        let head = Head {
            stops: vec![
                HeadStop {
                    id: 91,
                    center_x: 80.0,
                },
                HeadStop {
                    id: 92,
                    center_x: 250.0,
                },
                HeadStop {
                    id: 93,
                    center_x: 420.0,
                },
            ],
            column_x: vec![100.0, 400.0],
        };
        (page, head)
    }

    #[test]
    fn up_from_a_column_s_top_card_goes_to_the_featured_card_nearest_above() {
        let (page, head) = headed();
        assert_eq!(
            step_with_head(&page, &head, Some(1), Move::Up, Walk::Columns),
            Some(91)
        );
        assert_eq!(
            step_with_head(&page, &head, Some(11), Move::Up, Walk::Columns),
            Some(93)
        );
        assert_eq!(
            step_with_head(&page, &head, Some(12), Move::Up, Walk::Columns),
            Some(11),
            "inside a column, up is still the card above"
        );
    }

    #[test]
    fn left_and_right_walk_the_featured_cards_and_stop_at_their_ends() {
        let (page, head) = headed();
        assert_eq!(
            step_with_head(&page, &head, Some(91), Move::Right, Walk::Columns),
            Some(92)
        );
        assert_eq!(
            step_with_head(&page, &head, Some(93), Move::Left, Walk::Columns),
            Some(92)
        );
        assert_eq!(
            step_with_head(&page, &head, Some(91), Move::Left, Walk::Columns),
            None
        );
        assert_eq!(
            step_with_head(&page, &head, Some(93), Move::Right, Walk::Columns),
            None
        );
        assert_eq!(
            step_with_head(&page, &head, Some(92), Move::Up, Walk::Columns),
            None,
            "nothing above the featured cards"
        );
    }

    #[test]
    fn down_from_a_featured_card_goes_to_the_top_card_of_the_column_nearest_below() {
        let (page, head) = headed();
        assert_eq!(
            step_with_head(&page, &head, Some(91), Move::Down, Walk::Columns),
            Some(1)
        );
        assert_eq!(
            step_with_head(&page, &head, Some(93), Move::Down, Walk::Columns),
            Some(11)
        );
        let (mut empty_right, head) = headed();
        if let Some(column) = empty_right.columns.get_mut(1) {
            column.clear();
        }
        assert_eq!(
            step_with_head(&empty_right, &head, Some(93), Move::Down, Walk::Columns),
            Some(1),
            "a column with no cards is passed over"
        );
    }

    #[test]
    fn home_and_end_from_a_featured_card_go_to_the_first_and_last_card() {
        let (page, head) = headed();
        assert_eq!(
            step_with_head(&page, &head, Some(92), Move::First, Walk::Columns),
            Some(1)
        );
        assert_eq!(
            step_with_head(&page, &head, Some(92), Move::Last, Walk::Columns),
            Some(12)
        );
    }

    #[test]
    fn with_no_featured_cards_a_column_s_top_stays_put() {
        let (page, _) = headed();
        let none = Head::default();
        assert_eq!(
            step_with_head(&page, &none, Some(1), Move::Up, Walk::Columns),
            None
        );
        assert_eq!(
            step_with_head(&page, &none, Some(2), Move::Up, Walk::Columns),
            Some(1)
        );
    }
}
