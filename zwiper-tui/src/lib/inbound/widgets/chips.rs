//! A row of chips: pick one or several, move the focus with the arrows, toggle with Space or Enter.

use crossterm::event::KeyCode;
use ratatui::text::{Line, Span};

use crate::domain::theme::Palette;

/// How many chips can be picked at once.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Pick {
    /// Exactly one, like a tab or a currency.
    One,
    /// Any number, none included.
    Many,
}

/// A chip row's labels, picks and focus.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ChipRow {
    labels: Vec<String>,
    picked: Vec<bool>,
    focus: usize,
    pick: Pick,
}

impl ChipRow {
    /// A row with nothing picked, or for [`Pick::One`] the first chip.
    pub(crate) fn new(labels: &[&str], pick: Pick) -> Self {
        let mut picked = vec![false; labels.len()];
        if pick == Pick::One
            && let Some(first) = picked.first_mut()
        {
            *first = true;
        }
        Self {
            labels: labels.iter().map(|&l| l.to_owned()).collect(),
            picked,
            focus: 0,
            pick,
        }
    }

    /// Picks the chip at `index`, leaving it the only one in a [`Pick::One`] row.
    pub(crate) fn with_picked(mut self, index: usize) -> Self {
        self.pick_at(index);
        self
    }

    pub(crate) fn focus(&self) -> usize {
        self.focus
    }

    pub(crate) fn is_picked(&self, index: usize) -> bool {
        self.picked.get(index).copied().unwrap_or(false)
    }

    /// The picked chips' indexes, in order.
    pub(crate) fn picked(&self) -> Vec<usize> {
        self.picked
            .iter()
            .enumerate()
            .filter_map(|(i, &p)| p.then_some(i))
            .collect()
    }

    pub(crate) fn focus_next(&mut self) {
        self.focus = self
            .focus
            .saturating_add(1)
            .min(self.labels.len().saturating_sub(1));
    }

    pub(crate) fn focus_previous(&mut self) {
        self.focus = self.focus.saturating_sub(1);
    }

    fn pick_at(&mut self, index: usize) {
        if index >= self.picked.len() {
            return;
        }
        match self.pick {
            Pick::One => {
                for (i, picked) in self.picked.iter_mut().enumerate() {
                    *picked = i == index;
                }
            }
            Pick::Many => {
                if let Some(picked) = self.picked.get_mut(index) {
                    *picked = !*picked;
                }
            }
        }
    }

    /// Toggles the focused chip; in a [`Pick::One`] row, picks it.
    pub(crate) fn toggle(&mut self) {
        self.pick_at(self.focus);
    }

    /// Left and right move the focus, Space and Enter toggle. Returns whether the key was one of them.
    pub(crate) fn handle_key(&mut self, code: KeyCode) -> bool {
        match code {
            KeyCode::Left | KeyCode::Char('h') => self.focus_previous(),
            KeyCode::Right | KeyCode::Char('l') => self.focus_next(),
            KeyCode::Char(' ') | KeyCode::Enter => self.toggle(),
            _ => return false,
        }
        true
    }

    /// The row as one line: picked chips in the accent reversed, the rest muted, and the focused chip underlined while `focused`.
    pub(crate) fn line(&self, palette: &Palette, focused: bool) -> Line<'static> {
        let spans: Vec<Span<'static>> = self
            .labels
            .iter()
            .enumerate()
            .flat_map(|(i, label)| {
                let style = if self.is_picked(i) {
                    palette.accent.bold().reversed()
                } else {
                    palette.muted
                };
                let style = if focused && i == self.focus {
                    style.underlined()
                } else {
                    style
                };
                [Span::styled(format!(" {label} "), style), Span::raw(" ")]
            })
            .collect();
        Line::from(spans)
    }
}

#[cfg(test)]
mod tests {
    use ratatui::style::Modifier;

    use super::*;

    #[test]
    fn a_pick_one_row_keeps_exactly_one() {
        let mut row = ChipRow::new(&["USD", "EUR", "TIX"], Pick::One);
        assert_eq!(row.picked(), [0]);
        row.handle_key(KeyCode::Right);
        row.handle_key(KeyCode::Char(' '));
        assert_eq!(row.picked(), [1]);
        row.toggle();
        assert_eq!(row.picked(), [1], "picking it again keeps it");
        assert_eq!(row.with_picked(2).picked(), [2]);
    }

    #[test]
    fn a_pick_many_row_toggles_each_and_focus_stays_in_range() {
        let mut row = ChipRow::new(&["W", "U"], Pick::Many);
        assert_eq!(row.picked(), Vec::<usize>::new());
        row.handle_key(KeyCode::Enter);
        for _ in 0..5 {
            row.handle_key(KeyCode::Char('l'));
        }
        assert_eq!(row.focus(), 1);
        row.toggle();
        assert_eq!(row.picked(), [0, 1]);
        row.handle_key(KeyCode::Left);
        row.toggle();
        assert_eq!(row.picked(), [1]);
        assert!(!row.handle_key(KeyCode::Tab));
        assert!(!ChipRow::new(&[], Pick::One).is_picked(0));
    }

    #[test]
    fn picked_chips_reverse_and_the_focused_one_underlines() {
        let palette = Palette::terminal();
        let row = ChipRow::new(&["USD", "EUR"], Pick::One);
        let line = row.line(&palette, false);
        assert_eq!(line.to_string(), " USD   EUR  ");
        let first = &line.spans[0];
        assert!(first.style.add_modifier.contains(Modifier::REVERSED));
        assert!(!first.style.add_modifier.contains(Modifier::UNDERLINED));
        let focused = row.line(&palette, true);
        assert!(
            focused.spans[0]
                .style
                .add_modifier
                .contains(Modifier::UNDERLINED)
        );
    }
}
