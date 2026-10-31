//! The app's alert dialog: a title, a rule, a body, a rule, and a row of buttons with one focused, any of them in the danger color or disabled.

use crossterm::event::KeyCode;
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Wrap};

use super::{measure, panel};
use crate::domain::theme::Palette;

/// A dialog is at most this wide inside its panel, in cells.
const WIDTH: u16 = 56;

/// A button: its label, a letter that presses it from anywhere in the dialog, whether it is in the danger color, and whether it is disabled.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Button {
    pub(crate) label: String,
    pub(crate) key: Option<char>,
    pub(crate) danger: bool,
    pub(crate) disabled: bool,
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
}

impl AlertDialog {
    pub(crate) fn new(title: &str, body: &str) -> Self {
        Self {
            title: title.to_owned(),
            body: body.to_owned(),
            buttons: Vec::new(),
            focused: 0,
        }
    }

    /// Adds a button, pressed by Enter while focused or by `key` at any time.
    pub(crate) fn with_button(mut self, label: &str, key: Option<char>) -> Self {
        self.buttons.push(Button {
            label: label.to_owned(),
            key,
            danger: false,
            disabled: false,
        });
        self
    }

    /// Adds a button in the danger color, like the app's `Log out` and `Delete`.
    pub(crate) fn with_danger_button(mut self, label: &str, key: Option<char>) -> Self {
        self.buttons.push(Button {
            label: label.to_owned(),
            key,
            danger: true,
            disabled: false,
        });
        self
    }

    /// Focuses the button at `index`.
    pub(crate) fn with_focus(mut self, index: usize) -> Self {
        self.focused = index.min(self.buttons.len().saturating_sub(1));
        self
    }

    /// Relabels the button at `index` and enables or disables it, for a live label such as a countdown.
    pub(crate) fn set_button(&mut self, index: usize, label: &str, disabled: bool) {
        if let Some(button) = self.buttons.get_mut(index) {
            label.clone_into(&mut button.label);
            button.disabled = disabled;
        }
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
            KeyCode::Enter => self
                .buttons
                .get(self.focused)
                .filter(|b| !b.disabled)
                .map(|_| AlertChoice::Pressed(self.focused)),
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
                .position(|b| b.key == Some(c) && !b.disabled)
                .map(AlertChoice::Pressed),
            _ => None,
        }
    }

    /// The buttons as a row: the focused one reversed, a danger button in the error color, each with its letter.
    pub(crate) fn buttons_line(&self, palette: &Palette) -> Line<'static> {
        let spans: Vec<Span<'static>> = self
            .buttons
            .iter()
            .enumerate()
            .flat_map(|(i, button)| {
                let base: Style = if button.disabled {
                    palette.muted
                } else if button.danger {
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

    /// Draws the dialog centered over the screen: title, rule, body, rule, and the buttons.
    pub(crate) fn draw(&self, f: &mut Frame<'_>, palette: &Palette) {
        self.draw_with(f, palette, 0, |_, _| {});
    }

    /// [`AlertDialog::draw`] with `extra` rows under the body for the caller to fill, such as a password box, drawn by `fill` into that area.
    pub(crate) fn draw_with(
        &self,
        f: &mut Frame<'_>,
        palette: &Palette,
        extra: u16,
        fill: impl FnOnce(&mut Frame<'_>, Rect),
    ) {
        let body = Paragraph::new(self.body.clone()).wrap(Wrap { trim: true });
        let width = WIDTH.min(f.area().width.saturating_sub(4));
        let body_height = measure::height_of(body.line_count(width));
        let row = self.buttons_line(palette);
        // A row too wide for the dialog stacks its buttons, one per line.
        let buttons: Vec<Line<'static>> = if row.width() > usize::from(width) {
            row.spans
                .chunks(2)
                .filter_map(|pair| pair.first().cloned())
                .map(|span| Line::from(span).centered())
                .collect()
        } else {
            vec![row.centered()]
        };
        let buttons_height = measure::height_of(buttons.len());
        let inner = panel::draw(
            f,
            palette,
            f.area(),
            WIDTH,
            body_height
                .saturating_add(extra)
                .saturating_add(buttons_height)
                .saturating_add(3),
        );
        let [
            title_area,
            top_rule,
            body_area,
            extra_area,
            bottom_rule,
            buttons_area,
        ] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(body_height),
            Constraint::Length(extra),
            Constraint::Length(1),
            Constraint::Length(buttons_height),
        ])
        .areas(inner);
        f.render_widget(Line::from(self.title.clone()).bold().centered(), title_area);
        panel::rule(f, palette, top_rule);
        f.render_widget(body, body_area);
        fill(f, extra_area);
        panel::rule(f, palette, bottom_rule);
        f.render_widget(Paragraph::new(buttons), buttons_area);
    }
}

#[cfg(test)]
mod tests {

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
    fn a_disabled_button_cannot_be_pressed_until_enabled() {
        let mut alert = AlertDialog::new("Delete account", "Sure?")
            .with_button("Cancel", Some('n'))
            .with_danger_button("Delete (5)", Some('y'))
            .with_focus(1);
        alert.set_button(1, "Delete (5)", true);
        assert_eq!(alert.handle_key(KeyCode::Enter), None);
        assert_eq!(alert.handle_key(KeyCode::Char('y')), None);
        alert.set_button(1, "Delete", false);
        assert_eq!(alert.buttons[1].label, "Delete");
        assert_eq!(
            alert.handle_key(KeyCode::Enter),
            Some(AlertChoice::Pressed(1))
        );
    }
}
