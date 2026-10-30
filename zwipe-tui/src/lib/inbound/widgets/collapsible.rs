//! Collapsible sections: a header that opens and closes the lines under it, and an accordion that keeps at most one open.

use crossterm::event::KeyCode;
use ratatui::style::Stylize;
use ratatui::text::{Line, Span};

use crate::domain::theme::Palette;

/// Marks an open section's header.
const OPEN: &str = "▾ ";

/// Marks a closed section's header.
const CLOSED: &str = "▸ ";

/// One section's title and whether it is open.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Collapsible {
    pub(crate) title: String,
    pub(crate) open: bool,
}

impl Collapsible {
    /// A closed section.
    pub(crate) fn new(title: &str) -> Self {
        Self {
            title: title.to_owned(),
            open: false,
        }
    }

    pub(crate) fn toggle(&mut self) {
        self.open = !self.open;
    }

    /// The header: an arrow for open or closed, then the title, in the accent and bold while `focused`.
    pub(crate) fn header(&self, palette: &Palette, focused: bool) -> Line<'static> {
        let mark = if self.open { OPEN } else { CLOSED };
        let title = Span::styled(self.title.clone(), palette.accent);
        let line = Line::from(vec![Span::styled(mark, palette.muted), title]);
        if focused { line.bold() } else { line }
    }

    /// The header, then `body` indented under it when the section is open.
    pub(crate) fn lines(
        &self,
        palette: &Palette,
        focused: bool,
        body: Vec<Line<'static>>,
    ) -> Vec<Line<'static>> {
        let mut lines = vec![self.header(palette, focused)];
        if self.open {
            lines.extend(body.into_iter().map(|line| {
                let mut spans = vec![Span::raw("  ")];
                spans.extend(line.spans);
                Line::from(spans).style(line.style)
            }));
        }
        lines
    }
}

/// Sections where opening one closes the others, with a focused one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Accordion {
    pub(crate) sections: Vec<Collapsible>,
    pub(crate) focus: usize,
}

impl Accordion {
    /// Closed sections, the first focused.
    pub(crate) fn new(titles: &[&str]) -> Self {
        Self {
            sections: titles.iter().map(|t| Collapsible::new(t)).collect(),
            focus: 0,
        }
    }

    /// The open section's index, if any.
    pub(crate) fn open(&self) -> Option<usize> {
        self.sections.iter().position(|s| s.open)
    }

    /// Opens the focused section and closes the rest, or closes it when it is already open.
    pub(crate) fn toggle_focused(&mut self) {
        let focus = self.focus;
        for (i, section) in self.sections.iter_mut().enumerate() {
            section.open = i == focus && !section.open;
        }
    }

    /// j/k and the arrows move the focus, Enter and Space toggle. Returns whether the key was one of them.
    pub(crate) fn handle_key(&mut self, code: KeyCode) -> bool {
        match code {
            KeyCode::Down | KeyCode::Char('j') => {
                self.focus = self
                    .focus
                    .saturating_add(1)
                    .min(self.sections.len().saturating_sub(1));
            }
            KeyCode::Up | KeyCode::Char('k') => self.focus = self.focus.saturating_sub(1),
            KeyCode::Enter | KeyCode::Char(' ') => self.toggle_focused(),
            _ => return false,
        }
        true
    }

    /// Every section's lines in order, `body` giving each one's content by index.
    pub(crate) fn lines(
        &self,
        palette: &Palette,
        mut body: impl FnMut(usize) -> Vec<Line<'static>>,
    ) -> Vec<Line<'static>> {
        self.sections
            .iter()
            .enumerate()
            .flat_map(|(i, section)| {
                let content = if section.open { body(i) } else { Vec::new() };
                section.lines(palette, i == self.focus, content)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_section_shows_its_body_only_while_open() {
        let palette = Palette::terminal();
        let mut section = Collapsible::new("Rules");
        let body = || vec![Line::from("Draw a card.")];
        let closed = section.lines(&palette, false, body());
        assert_eq!(closed.len(), 1);
        assert_eq!(closed[0].to_string(), "▸ Rules");
        section.toggle();
        let open = section.lines(&palette, true, body());
        assert_eq!(open[0].to_string(), "▾ Rules");
        assert_eq!(open[1].to_string(), "  Draw a card.");
    }

    #[test]
    fn an_accordion_keeps_at_most_one_open() {
        let mut accordion = Accordion::new(&["Budget", "Mana", "Tags"]);
        accordion.handle_key(KeyCode::Enter);
        assert_eq!(accordion.open(), Some(0));
        accordion.handle_key(KeyCode::Char('j'));
        accordion.handle_key(KeyCode::Char(' '));
        assert_eq!(accordion.open(), Some(1), "opening Mana closes Budget");
        accordion.handle_key(KeyCode::Enter);
        assert_eq!(accordion.open(), None, "toggling the open one closes it");
        for _ in 0..5 {
            accordion.handle_key(KeyCode::Down);
        }
        assert_eq!(accordion.focus, 2);
        assert!(!accordion.handle_key(KeyCode::Tab));
    }

    #[test]
    fn accordion_lines_ask_only_for_the_open_body() {
        let mut accordion = Accordion::new(&["A", "B"]);
        accordion.toggle_focused();
        let mut asked = Vec::new();
        let lines = accordion.lines(&Palette::terminal(), |i| {
            asked.push(i);
            vec![Line::from("body")]
        });
        assert_eq!(asked, [0]);
        let text: Vec<String> = lines.iter().map(ToString::to_string).collect();
        assert_eq!(text, ["▾ A", "  body", "▸ B"]);
    }
}
