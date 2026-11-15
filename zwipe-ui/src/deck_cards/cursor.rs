//! The deck cards page's keyboard cursor (`context/plans/deck_keyboard_nav.md`): which card a
//! move lands on, given the cards as the page draws them, in columns. orangui-core's
//! [`grid_cursor`](orangui_core::grid_cursor) decides; here a card is named by its Scryfall id.
//! A screen of card lists side by side walks them as [`Walk::Lists`].

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
}
