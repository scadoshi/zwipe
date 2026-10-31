//! The app's screen header: the title centered and a faded `?` on the right when the screen has a hint to reopen. Help & feedback's `!` sits in the action bar, through [`crate::inbound::widgets::action_bar::with_help`].

use ratatui::Frame;
use ratatui::layout::{Margin, Rect};
use ratatui::style::Stylize;
use ratatui::text::Line;

use crate::domain::theme::Palette;

/// Whether the header carries the `?` that reopens the screen's hint.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Corner {
    /// The right corner stays empty.
    Empty,
    /// A `?` sits in the right corner.
    Hint,
}

/// Draws the header along the top row of `area`, spanning the frame's width less one column on each side, so the title and `?` sit in the same columns on every screen whatever margin the screen keeps.
pub(crate) fn draw(f: &mut Frame<'_>, palette: &Palette, area: Rect, title: &str, corner: Corner) {
    let frame = f.area().inner(Margin::new(1, 0));
    if area.height == 0 || frame.width == 0 {
        return;
    }
    let row = Rect {
        y: area.y,
        height: 1,
        ..frame
    };
    f.render_widget(
        Line::from(title.to_owned())
            .style(palette.accent)
            .bold()
            .centered(),
        row,
    );
    if corner == Corner::Hint {
        f.render_widget(Line::from("?").style(palette.muted).right_aligned(), row);
    }
}

#[cfg(test)]
mod tests {
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    use super::*;

    fn header(corner: Corner, width: u16) -> String {
        let mut terminal = Terminal::new(TestBackend::new(width, 1)).unwrap();
        terminal
            .draw(|f| draw(f, &Palette::terminal(), f.area(), "Account", corner))
            .unwrap();
        terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(ratatui::buffer::Cell::symbol)
            .collect()
    }

    #[test]
    fn the_title_sits_centered_with_only_the_hint_corner() {
        let row = header(Corner::Hint, 23);
        assert_eq!(row, "        Account      ? ");
        assert_eq!(header(Corner::Empty, 23).trim_end(), "        Account");
        header(Corner::Hint, 1);
        header(Corner::Hint, 2);
    }
}
