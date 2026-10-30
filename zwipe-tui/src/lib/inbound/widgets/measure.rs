//! Sizes of text in terminal cells.

use ratatui::layout::{Position, Rect};

/// A border on each side.
pub(crate) const BORDERS: u16 = 2;

/// How many cells `text` takes, one per character.
pub(crate) fn width_of(text: &str) -> u16 {
    u16::try_from(text.chars().count()).unwrap_or(u16::MAX)
}

/// A line count as a height in cells.
pub(crate) fn height_of(lines: usize) -> u16 {
    u16::try_from(lines).unwrap_or(u16::MAX)
}

/// Where the cursor sits after `text` in a bordered one-line box.
pub(crate) fn cursor_after(text: &str, area: Rect) -> Position {
    Position {
        x: area
            .x
            .saturating_add(1)
            .saturating_add(width_of(text))
            .min(area.right().saturating_sub(BORDERS)),
        y: area.y.saturating_add(1),
    }
}
