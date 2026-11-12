//! The deck page's List in two columns on a wide window: the groups in order, the first ones
//! down the left column and the rest down the right, split where the two hold the most even
//! share of rows. Two plain columns keep a change inside one group (a row opening, Art, a fold)
//! from laying out the whole list again, as a balanced CSS multi-column does.

/// The height a group's header counts for, in rows.
const HEADER_ROWS: usize = 1;

/// Where the right column starts among groups holding `rows` rows each, in order: the split
/// whose taller column is shortest, a header counting as [`HEADER_ROWS`] rows, the later split
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
}
