//! Collapsible sections: a header marking a section open or closed, and an accordion that keeps at most one open.

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

    /// The header: an arrow for open or closed, then the title, in the accent and bold while `focused`.
    pub(crate) fn header(&self, palette: &Palette, focused: bool) -> Line<'static> {
        let mark = if self.open { OPEN } else { CLOSED };
        let title = Span::styled(self.title.clone(), palette.accent);
        let line = Line::from(vec![Span::styled(mark, palette.muted), title]);
        if focused { line.bold() } else { line }
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_header_marks_open_and_closed() {
        let palette = Palette::terminal();
        let mut accordion = Accordion::new(&["Rules"]);
        let closed = accordion.sections[0].header(&palette, false);
        assert_eq!(closed.to_string(), "▸ Rules");
        accordion.toggle_focused();
        let open = accordion.sections[0].header(&palette, true);
        assert_eq!(open.to_string(), "▾ Rules");
    }

    #[test]
    fn an_accordion_keeps_at_most_one_open() {
        let mut accordion = Accordion::new(&["Budget", "Mana", "Tags"]);
        accordion.toggle_focused();
        assert_eq!(accordion.open(), Some(0));
        accordion.focus = 1;
        accordion.toggle_focused();
        assert_eq!(accordion.open(), Some(1), "opening Mana closes Budget");
        accordion.toggle_focused();
        assert_eq!(accordion.open(), None, "toggling the open one closes it");
    }
}
