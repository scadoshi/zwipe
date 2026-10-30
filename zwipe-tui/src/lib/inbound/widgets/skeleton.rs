//! Skeleton rows: ghost bars in the muted color that stand where content will load, with no text in them.

use ratatui::text::Line;

use crate::domain::theme::Palette;

/// The ghost block a skeleton bar is drawn with.
const GHOST: char = '░';

/// Each row's share of the width, in percent, repeating, so the rows read as uneven lines of text.
const WIDTHS: [usize; 4] = [72, 54, 86, 63];

/// `count` ghost rows for an area `width` cells wide.
pub(crate) fn rows(palette: &Palette, count: usize, width: usize) -> Vec<Line<'static>> {
    WIDTHS
        .iter()
        .cycle()
        .take(count)
        .map(|percent| {
            let cells = width.saturating_mul(*percent) / 100;
            Line::from(GHOST.to_string().repeat(cells.max(1))).style(palette.muted)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rows_are_ghosts_of_uneven_width_and_hold_no_text() {
        let rows = rows(&Palette::terminal(), 5, 50);
        assert_eq!(rows.len(), 5);
        let widths: Vec<usize> = rows.iter().map(Line::width).collect();
        assert_eq!(widths, [36, 27, 43, 31, 36]);
        assert!(
            rows.iter()
                .all(|r| r.to_string().chars().all(|c| c == GHOST))
        );
    }

    #[test]
    fn a_narrow_area_still_gets_a_ghost_per_row() {
        assert!(
            rows(&Palette::terminal(), 3, 0)
                .iter()
                .all(|r| r.width() == 1)
        );
    }
}
