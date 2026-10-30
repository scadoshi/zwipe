//! A framed panel centered over whatever is under it, for forms and dialogs.

use ratatui::Frame;
use ratatui::layout::{Constraint, Rect};
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
