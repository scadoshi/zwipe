//! The deck page's List in two columns on a wide window: the groups in order, the first ones
//! down the left column and the rest down the right, split where the two hold the most even
//! share of rows. Two plain columns keep a change inside one group (a row opening, Art, a fold)
//! from laying out the whole list again, as a balanced CSS multi-column does.
//!
//! The Stacks view lays its stacks in columns of their own heights, each stack under the one
//! above it in its column: two on a phone ([`masonry`]), as many as the page's width fits
//! ([`column_count`]) on a wide page ([`place`]).

/// The height a group's header counts for, in rows.
const HEADER_ROWS: usize = 1;

/// Where the right column starts among groups holding `rows` rows each, in order: the split
/// whose taller column is shortest, a header counting as `HEADER_ROWS` rows, the later split
/// on a tie, so the left column runs the longer as a page reads. A single group stays in the
/// left column; no groups split at 0.
#[must_use]
pub fn split_at(rows: &[usize]) -> usize {
    let heights: Vec<usize> = rows.iter().map(|n| n.saturating_add(HEADER_ROWS)).collect();
    let total: usize = heights.iter().copied().fold(0, usize::saturating_add);
    let mut best = (usize::MAX, rows.len().min(1));
    let mut left = 0_usize;
    for (at, height) in heights.iter().enumerate() {
        left = left.saturating_add(*height);
        let taller = left.max(total.saturating_sub(left));
        let split = at.saturating_add(1);
        if taller <= best.0 {
            best = (taller, split);
        }
    }
    best.1
}

/// A column of the phones' Stacks view.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Side {
    /// The left column.
    Left,
    /// The right column.
    Right,
}

/// A stack header's height with the space around it, in thousandths of the column's width: its
/// two lines (the name, then the price and cart) and its margins, about half a phone's column.
pub const STACK_HEADER: u32 = 500;

/// A whole card's height in thousandths of its width, from a card's 488 by 680 shape.
pub const STACK_CARD: u32 = 1393;

/// A tucked card's title strip, in thousandths of the column's width: the 13% of a card's height
/// its tuck leaves showing (`magic.css`).
pub const STACK_STRIP: u32 = 181;

/// A stack's height, in thousandths of its column's width, as [`masonry`] weighs it: its header,
/// then, unless `folded`, its first card whole and a strip for each card after it. A revealed card
/// counts for nothing, so a reveal never moves a stack to the other column.
#[must_use]
pub fn stack_height(cards: usize, folded: bool) -> u32 {
    if folded || cards == 0 {
        return STACK_HEADER;
    }
    let strips = u32::try_from(cards.saturating_sub(1)).unwrap_or(u32::MAX);
    STACK_HEADER
        .saturating_add(STACK_CARD)
        .saturating_add(STACK_STRIP.saturating_mul(strips))
}

/// The column each of a list of stacks, `heights` tall, goes in, in order: under the shorter
/// column so far, the left on a tie, so the groups read left to right and down as a page does
/// and each column holds its stacks in their order. The phones' two columns of [`place`].
#[must_use]
pub fn masonry(heights: &[u32]) -> Vec<Side> {
    place(heights, 2)
        .into_iter()
        .map(|column| if column == 0 { Side::Left } else { Side::Right })
        .collect()
}

/// The column, `0` the leftmost, each of a list of stacks, `heights` tall, goes in among
/// `columns` columns, in order: under the shortest column so far, the leftmost on a tie, so the
/// groups read left to right and down as a page does and each column holds its stacks in their
/// order. No room for a column still lays them in one.
#[must_use]
pub fn place(heights: &[u32], columns: usize) -> Vec<usize> {
    let mut totals = vec![0_u32; columns.max(1)];
    heights
        .iter()
        .map(|height| {
            let shortest = totals
                .iter()
                .enumerate()
                .min_by_key(|(at, total)| (**total, *at))
                .map_or(0, |(at, _)| at);
            if let Some(total) = totals.get_mut(shortest) {
                *total = total.saturating_add(*height);
            }
            shortest
        })
        .collect()
}

/// A stack column's narrowest width on a wide page, in hundredths of a rem: the 12.5rem the
/// stacks' grid fit as many of as it could.
pub const STACK_MIN_CENTIREM: u32 = 1250;

/// The room between two stack columns, in hundredths of a rem.
pub const STACK_GAP_CENTIREM: u32 = 125;

/// How many stack columns a page `width` hundredths of a rem across holds: as many
/// [`STACK_MIN_CENTIREM`]-wide columns as fit with [`STACK_GAP_CENTIREM`] between them, as the
/// grid's `auto-fill` placed them, and never fewer than one.
#[must_use]
pub fn column_count(width: u32) -> usize {
    let per = STACK_MIN_CENTIREM.saturating_add(STACK_GAP_CENTIREM);
    let fit = width
        .saturating_add(STACK_GAP_CENTIREM)
        .checked_div(per)
        .unwrap_or(1);
    usize::try_from(fit).unwrap_or(1).max(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_columns_split_where_their_rows_are_most_even() {
        assert_eq!(split_at(&[3, 3, 3, 3]), 2, "two groups each");
        assert_eq!(split_at(&[10, 1, 1]), 1, "one long group alone on the left");
        assert_eq!(
            split_at(&[1, 1, 10]),
            2,
            "one long group alone on the right"
        );
        assert_eq!(split_at(&[4]), 1, "a single group on the left");
        assert_eq!(split_at(&[]), 0, "no groups");
    }

    #[test]
    fn a_tie_keeps_the_left_column_the_taller() {
        assert_eq!(
            split_at(&[2, 2, 2]),
            2,
            "3 against 6 or 6 against 3: the left column longer"
        );
    }

    #[test]
    fn a_stack_is_its_header_its_first_card_whole_and_a_strip_per_card_after() {
        assert_eq!(stack_height(0, false), STACK_HEADER);
        assert_eq!(stack_height(1, false), STACK_HEADER + STACK_CARD);
        assert_eq!(
            stack_height(3, false),
            STACK_HEADER + STACK_CARD + 2 * STACK_STRIP
        );
        assert_eq!(
            stack_height(50, true),
            STACK_HEADER,
            "a folded stack is its header"
        );
    }

    #[test]
    fn each_stack_goes_under_the_shorter_column() {
        use Side::{Left, Right};
        let tall = stack_height(50, false);
        let short = stack_height(1, false);
        assert_eq!(
            masonry(&[tall, short, short, short]),
            [Left, Right, Right, Right],
            "the stacks after a long one fill the short column under each other"
        );
        assert_eq!(
            masonry(&[short, short, short, short]),
            [Left, Right, Left, Right]
        );
        assert_eq!(masonry(&[]), Vec::<Side>::new());
    }

    #[test]
    fn a_tie_goes_left_so_the_order_reads_left_then_right() {
        use Side::{Left, Right};
        let even = stack_height(2, false);
        assert_eq!(masonry(&[even, even]), [Left, Right]);
        assert_eq!(masonry(&[even]), [Left]);
    }

    #[test]
    fn a_folded_stack_counts_only_its_header() {
        use Side::{Left, Right};
        let folded = stack_height(40, true);
        let open = stack_height(3, false);
        assert_eq!(
            masonry(&[folded, open, open]),
            [Left, Right, Left],
            "the folded header leaves its column the shorter"
        );
    }

    #[test]
    fn stacks_go_under_the_shortest_of_any_number_of_columns() {
        let tall = stack_height(50, false);
        let short = stack_height(1, false);
        assert_eq!(
            place(&[tall, short, short, short, short], 3),
            [0, 1, 2, 1, 2],
            "the short stacks fill the two columns beside the long one"
        );
        assert_eq!(
            place(&[short, short, short, short], 4),
            [0, 1, 2, 3],
            "one row reads left to right"
        );
        assert_eq!(
            place(&[short, short], 1),
            [0, 0],
            "one column holds every stack"
        );
        assert_eq!(
            place(&[short, short], 0),
            [0, 0],
            "no room still lays one column"
        );
        assert_eq!(place(&[], 3), Vec::<usize>::new());
    }

    #[test]
    fn two_columns_place_as_the_phone_does() {
        let heights = [
            stack_height(30, false),
            stack_height(1, false),
            stack_height(2, true),
            stack_height(4, false),
        ];
        let sides: Vec<usize> = masonry(&heights)
            .into_iter()
            .map(|side| usize::from(side == Side::Right))
            .collect();
        assert_eq!(place(&heights, 2), sides);
    }

    #[test]
    fn the_column_count_follows_the_grid_it_replaces() {
        // 12.5rem columns with 1.25rem between them, in hundredths of a rem.
        assert_eq!(column_count(1250), 1, "one column's width");
        assert_eq!(column_count(1250 + 125 + 1249), 1, "a hair short of two");
        assert_eq!(column_count(1250 + 125 + 1250), 2, "exactly two");
        assert_eq!(column_count(8000), 5, "80rem: (80 + 1.25) / 13.75");
        assert_eq!(column_count(0), 1, "never fewer than one");
    }
}
