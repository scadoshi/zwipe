//! A list of items of different heights with one focused: the dictionary's tags, the printings, the card rows.
//!
//! The list pages instead of scrolling a line at a time. Pages fill greedily from the top with whole items, and the page holding the focused item is the one drawn, so moving the focus never shifts the items around it until it crosses a page edge. Nothing is kept between draws, which lets a screen draw from `&self`.

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Stylize;
use ratatui::text::Line;
use ratatui::widgets::{Paragraph, Wrap};

use super::measure::height_of;
use crate::domain::theme::Palette;

/// Drawn beside the focused item's first line.
const MARKER: &str = ">";

/// Columns kept left of every item for the marker.
const GUTTER: u16 = 2;

/// The first item of the page holding `focused`, when each page holds as many whole items as fit in `room` rows. An item taller than the room is a page of its own.
pub(crate) fn page_start(heights: &[u16], focused: usize, room: u16) -> usize {
    let mut start = 0;
    let mut used: u16 = 0;
    for (i, &height) in heights.iter().enumerate().take(focused.saturating_add(1)) {
        if used > 0 && used.saturating_add(height) > room {
            start = i;
            used = 0;
        }
        used = used.saturating_add(height);
    }
    start
}

/// How many rows `lines` take wrapped to `width` cells, at least one.
pub(crate) fn wrapped_height(lines: &[Line<'static>], width: u16) -> u16 {
    if width == 0 {
        return height_of(lines.len()).max(1);
    }
    let count = Paragraph::new(lines.to_vec())
        .wrap(Wrap { trim: false })
        .line_count(width);
    height_of(count).max(1)
}

/// Draws `items` into `area`, each wrapped and stacked, the page holding `focused` in view and a marker beside it.
pub(crate) fn draw(
    f: &mut Frame<'_>,
    palette: &Palette,
    area: Rect,
    items: &[Vec<Line<'static>>],
    focused: usize,
) {
    if area.width <= GUTTER || area.height == 0 {
        return;
    }
    let width = area.width.saturating_sub(GUTTER);
    let heights: Vec<u16> = items.iter().map(|i| wrapped_height(i, width)).collect();
    let start = page_start(&heights, focused, area.height);
    let mut y = area.y;
    for (i, (item, &height)) in items.iter().zip(&heights).enumerate().skip(start) {
        let room = area.bottom().saturating_sub(y);
        if room == 0 {
            break;
        }
        let rows = height.min(room);
        if i == focused {
            f.render_widget(
                Line::styled(MARKER, palette.accent).bold(),
                Rect::new(area.x, y, GUTTER, 1),
            );
        }
        f.render_widget(
            Paragraph::new(item.clone()).wrap(Wrap { trim: false }),
            Rect::new(area.x.saturating_add(GUTTER), y, width, rows),
        );
        y = y.saturating_add(rows);
    }
}

#[cfg(test)]
mod tests {
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    use super::*;

    fn text(items: &[Vec<Line<'static>>], focused: usize, width: u16, height: u16) -> String {
        let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
        terminal
            .draw(|f| draw(f, &Palette::terminal(), f.area(), items, focused))
            .unwrap();
        terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(ratatui::buffer::Cell::symbol)
            .collect()
    }

    fn items(count: usize) -> Vec<Vec<Line<'static>>> {
        (0..count)
            .map(|i| vec![Line::from(format!("item {i}")), Line::from("detail")])
            .collect()
    }

    #[test]
    fn pages_hold_whole_items_and_a_tall_one_stands_alone() {
        assert_eq!(page_start(&[2, 2, 2, 2], 0, 5), 0);
        assert_eq!(page_start(&[2, 2, 2, 2], 1, 5), 0);
        assert_eq!(page_start(&[2, 2, 2, 2], 2, 5), 2);
        assert_eq!(page_start(&[2, 2, 2, 2], 3, 5), 2);
        assert_eq!(page_start(&[1, 9, 1], 1, 4), 1);
        assert_eq!(page_start(&[1, 9, 1], 2, 4), 2);
        assert_eq!(page_start(&[], 3, 4), 0);
    }

    #[test]
    fn the_focused_page_shows_with_its_marker() {
        let screen = text(&items(6), 4, 20, 4);
        assert!(screen.contains("item 4"));
        assert!(screen.contains("item 5"));
        assert!(!screen.contains("item 3"));
        assert!(screen.contains("> item 4"));
    }

    #[test]
    fn long_lines_wrap_and_tiny_areas_draw_nothing() {
        let long = vec![Line::from("a line far too long for the width")];
        assert!(wrapped_height(&long, 10) > 1);
        assert_eq!(wrapped_height(&[], 10), 1);
        text(&items(3), 0, 1, 1);
        text(&items(3), 2, 3, 1);
    }
}
