//! The app's screen header: a faded `!` on the left for Help & feedback, the title centered, and a faded `?` on the right when the screen has a hint to reopen.

use ratatui::Frame;
use ratatui::layout::Rect;
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

/// Draws the header along the top row of `area`.
pub(crate) fn draw(f: &mut Frame<'_>, palette: &Palette, area: Rect, title: &str, corner: Corner) {
    if area.height == 0 || area.width == 0 {
        return;
    }
    let row = Rect { height: 1, ..area };
    f.render_widget(
        Line::from(title.to_owned())
            .style(palette.accent)
            .bold()
            .centered(),
        row,
    );
    f.render_widget(Line::from("!").style(palette.muted), row);
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
    fn the_title_sits_between_the_help_and_hint_corners() {
        let row = header(Corner::Hint, 21);
        assert_eq!(row, "!      Account      ?");
        assert_eq!(header(Corner::Empty, 21).trim_end(), "!      Account");
        header(Corner::Hint, 1);
    }
}
