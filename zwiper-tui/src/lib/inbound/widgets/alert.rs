//! The app's alert dialog: a title, a rule, a body, a rule, and a row of buttons with one focused, any of them in the danger color or disabled.

use crossterm::event::KeyCode;
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Stylize;
use ratatui::text::Line;
use ratatui::widgets::{Paragraph, Wrap};

use super::{button, measure, panel};
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

    /// Each button's label as it shows, with its letter.
    fn labels(&self) -> Vec<String> {
        self.buttons
            .iter()
            .map(|button| match button.key {
                Some(k) => format!("{} ({k})", button.label),
                None => button.label.clone(),
            })
            .collect()
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
        let labels = self.labels();
        let row: Vec<button::Button<'_>> = self
            .buttons
            .iter()
            .zip(&labels)
            .map(|(b, label)| {
                let shown = button::Button::new(label).disabled(b.disabled);
                if b.danger { shown.danger() } else { shown }
            })
            .collect();
        let focused = Some(self.focused);
        // One line, or one button per line when the row is wider than the dialog, as Help's three are on a narrow screen.
        let stacked = button::row_width(&row) > width;
        let buttons_height = if stacked {
            measure::height_of(row.len())
        } else {
            1
        };
        let inner = panel::dialog(
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
        if stacked {
            let lines: Vec<Line<'static>> = row
                .iter()
                .enumerate()
                .map(|(i, b)| Line::from(button::spans(palette, b, focused == Some(i))).centered())
                .collect();
            f.render_widget(Paragraph::new(lines), buttons_area);
        } else {
            button::draw_row(f, palette, buttons_area, &row, focused);
        }
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

    /// The dialog drawn at `width` by `height`, row by row.
    fn rows(alert: &AlertDialog, width: u16, height: u16) -> Vec<String> {
        let palette = Palette::for_account("gruvbox", true);
        let mut terminal =
            ratatui::Terminal::new(ratatui::backend::TestBackend::new(width, height)).unwrap();
        terminal.draw(|f| alert.draw(f, &palette)).unwrap();
        let buffer = terminal.backend().buffer();
        (0..height)
            .map(|y| (0..width).map(|x| buffer[(x, y)].symbol()).collect())
            .collect()
    }

    #[test]
    fn the_buttons_are_bracketed_on_one_line_at_every_size() {
        for (width, height) in crate::test_support::BUTTON_SIZES {
            let rows = rows(&log_out(), width, height);
            let found: Vec<&String> = rows.iter().filter(|r| r.contains("[ ")).collect();
            assert_eq!(found.len(), 1, "one button line at {width}x{height}");
            assert!(
                found[0].contains("[ Log out (y) ]  [ Cancel (n) ]"),
                "at {width}x{height}: {}",
                found[0]
            );
            assert!(!rows.iter().any(|r| r.contains("│ Log out")), "no box");
        }
    }

    #[test]
    fn a_row_wider_than_the_dialog_stacks_one_button_per_line() {
        let help = super::super::help::dialog();
        let rows = rows(&help, 60, 20);
        let lines: Vec<&String> = rows.iter().filter(|r| r.contains("[ ")).collect();
        assert_eq!(lines.len(), 3, "stacked:\n{}", rows.join("\n"));
        assert!(lines[0].contains("[ Close (c) ]"));
    }
}
