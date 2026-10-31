//! The app's alert dialog: a title, a body, and a row of buttons with one focused, optionally in the danger style.

use crossterm::event::KeyCode;
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout};
use ratatui::style::{Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Wrap};

use super::measure::height_of;
use super::panel;
use crate::domain::theme::Palette;

/// A dialog is at most this wide inside its panel, in cells.
const WIDTH: u16 = 56;

/// Whether the dialog confirms something that cannot be undone.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Tone {
    #[default]
    Normal,
    /// The title and the first button in the error color.
    Danger,
}

/// A button: its label, a letter that presses it from anywhere in the dialog, and whether it is in the danger color.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Button {
    pub(crate) label: String,
    pub(crate) key: Option<char>,
    pub(crate) danger: bool,
}

/// What a key did to the dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AlertChoice {
    /// The button at this index was pressed.
    Pressed(usize),
    /// Esc closed the dialog without a choice.
    Dismissed,
}

/// An alert over the screen. The first button is focused when it opens.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AlertDialog {
    pub(crate) title: String,
    pub(crate) body: String,
    pub(crate) buttons: Vec<Button>,
    pub(crate) focused: usize,
    pub(crate) tone: Tone,
}

impl AlertDialog {
    pub(crate) fn new(title: &str, body: &str) -> Self {
        Self {
            title: title.to_owned(),
            body: body.to_owned(),
            buttons: Vec::new(),
            focused: 0,
            tone: Tone::Normal,
        }
    }

    /// Adds a button, pressed by Enter while focused or by `key` at any time.
    pub(crate) fn with_button(mut self, label: &str, key: Option<char>) -> Self {
        self.buttons.push(Button {
            label: label.to_owned(),
            key,
            danger: false,
        });
        self
    }

    /// Adds a button in the danger color, like the app's danger confirm in Clear skips.
    pub(crate) fn with_danger_button(mut self, label: &str, key: Option<char>) -> Self {
        self.buttons.push(Button {
            label: label.to_owned(),
            key,
            danger: true,
        });
        self
    }

    pub(crate) fn with_tone(mut self, tone: Tone) -> Self {
        self.tone = tone;
        self
    }

    pub(crate) fn focus_next(&mut self) {
        self.focused = self
            .focused
            .saturating_add(1)
            .min(self.buttons.len().saturating_sub(1));
    }

    pub(crate) fn focus_previous(&mut self) {
        self.focused = self.focused.saturating_sub(1);
    }

    /// Moves the focus or presses a button. `None` while the dialog stays open with no choice.
    pub(crate) fn handle_key(&mut self, code: KeyCode) -> Option<AlertChoice> {
        match code {
            KeyCode::Esc => Some(AlertChoice::Dismissed),
            KeyCode::Enter => {
                (!self.buttons.is_empty()).then_some(AlertChoice::Pressed(self.focused))
            }
            KeyCode::Right | KeyCode::Tab | KeyCode::Char('l') => {
                self.focus_next();
                None
            }
            KeyCode::Left | KeyCode::BackTab | KeyCode::Char('h') => {
                self.focus_previous();
                None
            }
            KeyCode::Char(c) => self
                .buttons
                .iter()
                .position(|b| b.key == Some(c))
                .map(AlertChoice::Pressed),
            _ => None,
        }
    }

    /// The buttons as a row: the focused one reversed, a danger dialog's first one in the error color, each with its letter.
    pub(crate) fn buttons_line(&self, palette: &Palette) -> Line<'static> {
        let spans: Vec<Span<'static>> = self
            .buttons
            .iter()
            .enumerate()
            .flat_map(|(i, button)| {
                let base: Style = if button.danger || (i == 0 && self.tone == Tone::Danger) {
                    palette.error
                } else {
                    palette.accent
                };
                let style = if i == self.focused {
                    base.bold().reversed()
                } else {
                    base
                };
                let key = button.key.map_or_else(String::new, |k| format!(" ({k})"));
                [
                    Span::styled(format!(" {}{key} ", button.label), style),
                    Span::raw("  "),
                ]
            })
            .collect();
        Line::from(spans)
    }

    /// Draws the dialog centered over the screen.
    pub(crate) fn draw(&self, f: &mut Frame<'_>, palette: &Palette) {
        let body = Paragraph::new(self.body.clone()).wrap(Wrap { trim: true });
        let body_height = height_of(body.line_count(WIDTH));
        let inner = panel::draw(f, palette, f.area(), WIDTH, body_height.saturating_add(4));
        let [title_area, body_area, _, buttons_area] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(body_height.saturating_add(1)),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .areas(inner);
        let title_style = match self.tone {
            Tone::Normal => Style::new(),
            Tone::Danger => palette.error,
        };
        f.render_widget(
            Line::from(self.title.clone()).style(title_style).bold(),
            title_area,
        );
        f.render_widget(body, body_area);
        f.render_widget(self.buttons_line(palette), buttons_area);
    }
}

#[cfg(test)]
mod tests {
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    use super::*;

    fn log_out() -> AlertDialog {
        AlertDialog::new("Log out", "Are you sure you want to log out?")
            .with_button("Log out", Some('y'))
            .with_button("Cancel", Some('n'))
    }

    #[test]
    fn enter_presses_the_focused_button_and_the_arrows_move_it() {
        let mut alert = log_out();
        assert_eq!(
            alert.handle_key(KeyCode::Enter),
            Some(AlertChoice::Pressed(0))
        );
        assert_eq!(alert.handle_key(KeyCode::Right), None);
        assert_eq!(alert.handle_key(KeyCode::Right), None, "stops at the last");
        assert_eq!(
            alert.handle_key(KeyCode::Enter),
            Some(AlertChoice::Pressed(1))
        );
        alert.handle_key(KeyCode::Left);
        assert_eq!(alert.focused, 0);
    }

    #[test]
    fn letters_press_their_button_and_esc_dismisses() {
        let mut alert = log_out();
        assert_eq!(
            alert.handle_key(KeyCode::Char('n')),
            Some(AlertChoice::Pressed(1))
        );
        assert_eq!(
            alert.handle_key(KeyCode::Char('y')),
            Some(AlertChoice::Pressed(0))
        );
        assert_eq!(alert.handle_key(KeyCode::Char('z')), None);
        assert_eq!(alert.handle_key(KeyCode::Esc), Some(AlertChoice::Dismissed));
    }

    #[test]
    fn a_danger_alert_draws_its_title_and_first_button_in_the_error_color() {
        let palette = Palette::for_account("gruvbox", true);
        let alert = AlertDialog::new("Delete account", "This cannot be undone.")
            .with_button("Delete", None)
            .with_button("Cancel", Some('n'))
            .with_tone(Tone::Danger);
        let mut terminal = Terminal::new(TestBackend::new(70, 12)).unwrap();
        terminal.draw(|f| alert.draw(f, &palette)).unwrap();
        let buffer = terminal.backend().buffer();
        let text: String = buffer
            .content()
            .iter()
            .map(ratatui::buffer::Cell::symbol)
            .collect();
        assert!(text.contains("Delete account"));
        assert!(text.contains("This cannot be undone."));
        assert!(text.contains(" Cancel (n) "));
        let error = palette.error.fg.unwrap();
        let red_cells = buffer
            .content()
            .iter()
            .filter(|c| c.fg == error || c.bg == error)
            .count();
        assert!(red_cells >= "Delete account".len() + " Delete ".len());
    }
}
