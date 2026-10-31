//! The "?" info popup: a title over a list of bullet points, closed with Esc, Enter, q or ? again.

use crossterm::event::KeyCode;
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout};
use ratatui::style::Stylize;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Wrap};

use super::measure::height_of;
use super::{action_bar, panel};
use crate::domain::theme::Palette;

/// The popup is at most this wide inside its panel, in cells.
const WIDTH: u16 = 56;

/// Starts each point.
const BULLET: &str = "• ";

/// Whether `code` closes an info popup.
pub(crate) fn closes_on(code: KeyCode) -> bool {
    matches!(
        code,
        KeyCode::Esc | KeyCode::Enter | KeyCode::Char('q' | '?')
    )
}

/// An info popup's content.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct InfoPopup {
    pub(crate) title: String,
    pub(crate) bullets: Vec<String>,
}

impl InfoPopup {
    pub(crate) fn new(title: &str, bullets: &[&str]) -> Self {
        Self {
            title: title.to_owned(),
            bullets: bullets.iter().map(|&b| b.to_owned()).collect(),
        }
    }

    fn lines(&self, palette: &Palette) -> Vec<Line<'static>> {
        self.bullets
            .iter()
            .map(|bullet| {
                Line::from(vec![
                    Span::styled(BULLET, palette.accent),
                    Span::raw(bullet.clone()),
                ])
            })
            .collect()
    }

    /// Draws the popup centered over the screen.
    pub(crate) fn draw(&self, f: &mut Frame<'_>, palette: &Palette) {
        let body = Paragraph::new(self.lines(palette)).wrap(Wrap { trim: true });
        let body_height = height_of(body.line_count(WIDTH));
        let inner = panel::draw(f, palette, f.area(), WIDTH, body_height.saturating_add(4));
        let [title_area, body_area, _, keys_area] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(body_height.saturating_add(1)),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .areas(inner);
        f.render_widget(
            Line::from(self.title.clone()).style(palette.accent).bold(),
            title_area,
        );
        f.render_widget(body, body_area);
        f.render_widget(action_bar::line(palette, &[("Esc", "close")]), keys_area);
    }
}

#[cfg(test)]
mod tests {
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    use super::*;

    #[test]
    fn the_popup_lists_each_point_as_a_bullet() {
        let info = InfoPopup::new("Swiping", &["Right adds the card", "Left passes"]);
        let mut terminal = Terminal::new(TestBackend::new(70, 14)).unwrap();
        terminal
            .draw(|f| info.draw(f, &Palette::terminal()))
            .unwrap();
        let screen: String = terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(ratatui::buffer::Cell::symbol)
            .collect();
        assert!(screen.contains("Swiping"));
        assert!(screen.contains("• Right adds the card"));
        assert!(screen.contains("• Left passes"));
        assert!(screen.contains("Esc close"));
    }

    #[test]
    fn esc_enter_q_and_question_mark_close_it() {
        for code in [
            KeyCode::Esc,
            KeyCode::Enter,
            KeyCode::Char('q'),
            KeyCode::Char('?'),
        ] {
            assert!(closes_on(code), "{code:?}");
        }
        assert!(!closes_on(KeyCode::Char('j')));
    }
}
