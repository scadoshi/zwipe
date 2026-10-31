//! A bottom sheet of buttons, like the app's "More actions" sheets: one focused, some disabled, some in the danger style.

use crossterm::event::KeyCode;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};

use super::sheet::Sheet;
use crate::domain::theme::Palette;

/// How a menu button looks.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ItemTone {
    #[default]
    Normal,
    /// In the error color, for actions that cannot be undone.
    Danger,
}

/// One button: its label, whether it can be pressed, and its tone.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MenuItem {
    pub(crate) label: String,
    pub(crate) enabled: bool,
    pub(crate) tone: ItemTone,
}

impl MenuItem {
    pub(crate) fn new(label: &str) -> Self {
        Self {
            label: label.to_owned(),
            enabled: true,
            tone: ItemTone::Normal,
        }
    }

    pub(crate) fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub(crate) fn danger(mut self) -> Self {
        self.tone = ItemTone::Danger;
        self
    }
}

/// What a key did to the menu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MenuKey {
    /// The focus moved, or nothing happened.
    Stay,
    /// The enabled button at this index was pressed.
    Pressed(usize),
    /// Esc or q closed the menu.
    Close,
}

/// A titled sheet of buttons with one focused.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Menu {
    pub(crate) title: String,
    pub(crate) items: Vec<MenuItem>,
    pub(crate) focus: usize,
}

impl Menu {
    pub(crate) fn new(title: &str, items: Vec<MenuItem>) -> Self {
        Self {
            title: title.to_owned(),
            items,
            focus: 0,
        }
    }

    /// j/k and the arrows move, Enter and Space press the focused button when it is enabled, Esc and q close.
    pub(crate) fn handle_key(&mut self, code: KeyCode) -> MenuKey {
        match code {
            KeyCode::Esc | KeyCode::Char('q') => MenuKey::Close,
            KeyCode::Down | KeyCode::Char('j') | KeyCode::Tab => {
                self.focus = self
                    .focus
                    .saturating_add(1)
                    .min(self.items.len().saturating_sub(1));
                MenuKey::Stay
            }
            KeyCode::Up | KeyCode::Char('k') | KeyCode::BackTab => {
                self.focus = self.focus.saturating_sub(1);
                MenuKey::Stay
            }
            KeyCode::Enter | KeyCode::Char(' ') => match self.items.get(self.focus) {
                Some(item) if item.enabled => MenuKey::Pressed(self.focus),
                _ => MenuKey::Stay,
            },
            _ => MenuKey::Stay,
        }
    }

    fn lines(&self, palette: &Palette) -> Vec<Line<'static>> {
        self.items
            .iter()
            .enumerate()
            .map(|(i, item)| {
                let base: Style = match (item.enabled, item.tone) {
                    (false, _) => palette.muted,
                    (true, ItemTone::Danger) => palette.error,
                    (true, ItemTone::Normal) => palette.accent,
                };
                let focused = i == self.focus;
                let style = if focused {
                    base.bold().reversed()
                } else {
                    base
                };
                Line::from(vec![
                    Span::raw(if focused { "> " } else { "  " }),
                    Span::styled(format!(" {} ", item.label), style),
                ])
            })
            .collect()
    }

    /// Draws the menu as a sheet over the bottom of `area`, with a Close footer.
    pub(crate) fn draw(&self, f: &mut Frame<'_>, palette: &Palette, area: Rect) {
        Sheet::new(&self.title, self.lines(palette))
            .with_footer("Esc", "Close")
            .draw(f, palette, area);
    }
}

#[cfg(test)]
mod tests {
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    use super::*;

    fn more() -> Menu {
        Menu::new(
            "More actions",
            vec![
                MenuItem::new("Import cards"),
                MenuItem::new("Export cards").enabled(false),
                MenuItem::new("Delete deck").danger(),
            ],
        )
    }

    #[test]
    fn disabled_buttons_do_not_press_and_the_focus_stays_in_range() {
        let mut menu = more();
        assert_eq!(menu.handle_key(KeyCode::Enter), MenuKey::Pressed(0));
        menu.handle_key(KeyCode::Char('j'));
        assert_eq!(menu.handle_key(KeyCode::Enter), MenuKey::Stay);
        for _ in 0..5 {
            menu.handle_key(KeyCode::Down);
        }
        assert_eq!(menu.handle_key(KeyCode::Char(' ')), MenuKey::Pressed(2));
        menu.handle_key(KeyCode::Up);
        assert_eq!(menu.focus, 1);
        assert_eq!(menu.handle_key(KeyCode::Esc), MenuKey::Close);
    }

    #[test]
    fn the_menu_draws_its_title_buttons_and_close() {
        let mut terminal = Terminal::new(TestBackend::new(40, 12)).unwrap();
        terminal
            .draw(|f| more().draw(f, &Palette::terminal(), f.area()))
            .unwrap();
        let text: String = terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(ratatui::buffer::Cell::symbol)
            .collect();
        for expected in ["More actions", "Import cards", "Delete deck", "Close"] {
            assert!(text.contains(expected), "missing {expected}");
        }
    }
}
