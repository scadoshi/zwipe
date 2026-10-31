//! A framed panel centered over whatever is under it, for forms and dialogs, and the full-bleed rules that divide one.

use ratatui::Frame;
use ratatui::layout::{Constraint, Rect};
use ratatui::text::Line;
use ratatui::widgets::{Block, Clear, Padding};

use super::measure::BORDERS;
use crate::domain::theme::Palette;

/// Clears and frames a centered panel of the given inner size, and returns the area inside its border and padding.
pub(crate) fn draw(
    f: &mut Frame<'_>,
    palette: &Palette,
    area: Rect,
    width: u16,
    height: u16,
) -> Rect {
    let outer = area.centered(
        Constraint::Max(width.saturating_add(BORDERS).saturating_add(BORDERS)),
        Constraint::Length(height.saturating_add(BORDERS)),
    );
    let block = Block::bordered()
        .border_style(palette.border)
        .style(palette.base)
        .padding(Padding::horizontal(1));
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
                let inner = draw(f, &palette, f.area(), 10, 3);
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
