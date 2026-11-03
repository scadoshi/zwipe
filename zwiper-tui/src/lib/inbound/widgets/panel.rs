//! The frames every box in the app draws with, a centered panel on the grid or over the screen, and the full-bleed rules that divide one.
//!
//! A box that sits on the grid backdrop is `bordered`: border and title in their colors and no fill, so the tiles show through under its text. A box drawn over other content (a dialog, sheet, toast or popup) is an `overlay`: filled with the theme's background, so nothing under it shows through its text.

use ratatui::Frame;
use ratatui::layout::{Constraint, Rect};
use ratatui::text::Line;
use ratatui::widgets::{Block, BorderType, Clear, Padding};

use super::measure::BORDERS;
use crate::domain::theme::Palette;

/// A box on the grid: a border on every side with rounded corners and no background, so the grid's tiles show under its text. Its text draws in the theme's text color unless a span says otherwise.
pub(crate) fn bordered(palette: &Palette) -> Block<'static> {
    Block::bordered()
        .border_type(BorderType::Rounded)
        .style(palette.text())
}

/// A box drawn over other content, a dialog, sheet, toast or popup: [`bordered`]'s frame filled with the theme's background, so what lies under it never shows through its text. The caller clears the area first with a `Clear`.
pub(crate) fn overlay(palette: &Palette) -> Block<'static> {
    Block::bordered()
        .border_type(BorderType::Rounded)
        .style(palette.base)
}

/// The area of a centered panel whose inside is `width` by `height`, with room for its border and padding.
fn centered_area(area: Rect, width: u16, height: u16) -> Rect {
    area.centered(
        Constraint::Max(width.saturating_add(BORDERS).saturating_add(BORDERS)),
        Constraint::Length(height.saturating_add(BORDERS)),
    )
}

/// The frame of a centered panel, in the border color and padded a cell each side.
fn panel_block(block: Block<'static>, palette: &Palette) -> Block<'static> {
    block
        .border_style(palette.border)
        .padding(Padding::horizontal(1))
}

/// Frames a centered panel of the given inner size on the grid, with no fill, and returns the area inside its border and padding.
pub(crate) fn centered(
    f: &mut Frame<'_>,
    palette: &Palette,
    area: Rect,
    width: u16,
    height: u16,
) -> Rect {
    let outer = centered_area(area, width, height);
    let block = panel_block(bordered(palette), palette);
    let inner = block.inner(outer);
    f.render_widget(block, outer);
    inner
}

/// Clears and frames a centered dialog of the given inner size over whatever is under it, filled as an [`overlay`], and returns the area inside its border and padding.
pub(crate) fn dialog(
    f: &mut Frame<'_>,
    palette: &Palette,
    area: Rect,
    width: u16,
    height: u16,
) -> Rect {
    let outer = centered_area(area, width, height);
    let block = panel_block(overlay(palette), palette);
    let inner = block.inner(outer);
    f.render_widget(Clear, outer);
    f.render_widget(block, outer);
    inner
}

/// Draws a full-bleed rule along `row`, a row inside a panel's one-cell padding, meeting the border on both sides.
pub(crate) fn rule(f: &mut Frame<'_>, palette: &Palette, row: Rect) {
    if row.height == 0 {
        return;
    }
    let left = row.x.saturating_sub(2);
    let width = row
        .width
        .saturating_add(4)
        .min(f.area().right().saturating_sub(left));
    let dashes = "─".repeat(usize::from(width.saturating_sub(2)));
    let area = Rect {
        x: left,
        y: row.y,
        width,
        height: 1,
    };
    f.render_widget(
        Line::from(format!("├{dashes}┤")).style(palette.border),
        area,
    );
}

#[cfg(test)]
mod tests {
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;
    use ratatui::layout::{Constraint, Layout};

    use super::*;

    #[test]
    fn a_rule_meets_the_border_on_both_sides() {
        let mut terminal = Terminal::new(TestBackend::new(20, 5)).unwrap();
        terminal
            .draw(|f| {
                let palette = Palette::terminal();
                let inner = dialog(f, &palette, f.area(), 10, 3);
                let [_, middle, _] = Layout::vertical([Constraint::Length(1); 3]).areas(inner);
                rule(f, &palette, middle);
            })
            .unwrap();
        let buffer = terminal.backend().buffer();
        let row: String = (0..20)
            .map(|x| buffer.cell((x, 2)).unwrap().symbol())
            .collect();
        assert_eq!(row.trim(), "├────────────┤");
    }
}
