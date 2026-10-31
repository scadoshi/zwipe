//! A text box of many lines, for pasting a decklist: typing, Enter for a new line, Backspace joining lines, the arrows moving the cursor, and a placeholder of several lines while empty.

use crossterm::event::KeyCode;
use ratatui::Frame;
use ratatui::layout::{Position, Rect};
use ratatui::text::Line;
use ratatui::widgets::{Block, Paragraph};

use crate::domain::theme::Palette;

/// A multi-line text box's state. The cursor counts characters and always sits inside the text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TextArea {
    /// Never empty: an empty box has one empty line.
    lines: Vec<String>,
    row: usize,
    column: usize,
    placeholder: Vec<String>,
}

impl Default for TextArea {
    fn default() -> Self {
        Self {
            lines: vec![String::new()],
            row: 0,
            column: 0,
            placeholder: Vec::new(),
        }
    }
}

impl TextArea {
    /// Lines shown muted while the box is empty.
    pub(crate) fn with_placeholder(mut self, lines: &[&str]) -> Self {
        self.placeholder = lines.iter().map(|&l| l.to_owned()).collect();
        self
    }

    /// The text, lines joined by newlines.
    pub(crate) fn text(&self) -> String {
        self.lines.join("\n")
    }

    /// Whether the text is blank once trimmed.
    pub(crate) fn is_blank(&self) -> bool {
        self.lines.iter().all(|l| l.trim().is_empty())
    }

    /// The cursor as a line and a character within it.
    #[cfg(test)]
    pub(crate) fn cursor(&self) -> (usize, usize) {
        (self.row, self.column)
    }

    fn line_len(&self, row: usize) -> usize {
        self.lines.get(row).map_or(0, |l| l.chars().count())
    }

    /// The byte offset of character `column` in line `row`, or the line's end.
    fn byte_at(&self, row: usize, column: usize) -> usize {
        self.lines.get(row).map_or(0, |line| {
            line.char_indices()
                .nth(column)
                .map_or(line.len(), |(i, _)| i)
        })
    }

    fn insert(&mut self, c: char) {
        let at = self.byte_at(self.row, self.column);
        if let Some(line) = self.lines.get_mut(self.row) {
            line.insert(at, c);
            self.column = self.column.saturating_add(1);
        }
    }

    fn newline(&mut self) {
        let at = self.byte_at(self.row, self.column);
        let Some(line) = self.lines.get_mut(self.row) else {
            return;
        };
        let rest = line.split_off(at);
        self.row = self.row.saturating_add(1);
        self.lines.insert(self.row, rest);
        self.column = 0;
    }

    fn backspace(&mut self) {
        if self.column > 0 {
            let before = self.column.saturating_sub(1);
            let at = self.byte_at(self.row, before);
            if let Some(line) = self.lines.get_mut(self.row) {
                line.remove(at);
            }
            self.column = before;
        } else if let Some(above) = self.row.checked_sub(1) {
            let line = self.lines.remove(self.row);
            self.column = self.line_len(above);
            if let Some(previous) = self.lines.get_mut(above) {
                previous.push_str(&line);
            }
            self.row = above;
        }
    }

    fn delete(&mut self) {
        if self.column < self.line_len(self.row) {
            let at = self.byte_at(self.row, self.column);
            if let Some(line) = self.lines.get_mut(self.row) {
                line.remove(at);
            }
        } else if self.row.saturating_add(1) < self.lines.len() {
            let next = self.lines.remove(self.row.saturating_add(1));
            if let Some(line) = self.lines.get_mut(self.row) {
                line.push_str(&next);
            }
        }
    }

    /// Edits for typing, Enter, Backspace, Delete, the arrows, Home and End. Returns whether the key was one of them.
    pub(crate) fn handle_key(&mut self, code: KeyCode) -> bool {
        match code {
            KeyCode::Char(c) => self.insert(c),
            KeyCode::Enter => self.newline(),
            KeyCode::Backspace => self.backspace(),
            KeyCode::Delete => self.delete(),
            KeyCode::Left => {
                if self.column > 0 {
                    self.column = self.column.saturating_sub(1);
                } else if let Some(above) = self.row.checked_sub(1) {
                    self.row = above;
                    self.column = self.line_len(above);
                }
            }
            KeyCode::Right => {
                if self.column < self.line_len(self.row) {
                    self.column = self.column.saturating_add(1);
                } else if self.row.saturating_add(1) < self.lines.len() {
                    self.row = self.row.saturating_add(1);
                    self.column = 0;
                }
            }
            KeyCode::Up => {
                self.row = self.row.saturating_sub(1);
                self.column = self.column.min(self.line_len(self.row));
            }
            KeyCode::Down => {
                self.row = self
                    .row
                    .saturating_add(1)
                    .min(self.lines.len().saturating_sub(1));
                self.column = self.column.min(self.line_len(self.row));
            }
            KeyCode::Home => self.column = 0,
            KeyCode::End => self.column = self.line_len(self.row),
            _ => return false,
        }
        true
    }

    /// Lines scrolled off the top of a box `height` rows tall, so the cursor stays in view.
    fn skip(&self, height: u16) -> usize {
        let inner = usize::from(height.saturating_sub(2));
        self.row.saturating_sub(inner.saturating_sub(1))
    }

    /// Draws the box titled `title` in `area`, its border in the focus color while `focused`.
    pub(crate) fn draw(
        &self,
        f: &mut Frame<'_>,
        palette: &Palette,
        area: Rect,
        title: &str,
        focused: bool,
    ) {
        let border = if focused { palette.add } else { palette.border };
        let lines: Vec<Line<'static>> = if self.lines.len() == 1 && self.line_len(0) == 0 {
            self.placeholder
                .iter()
                .map(|l| Line::from(l.clone()).style(palette.muted))
                .collect()
        } else {
            self.lines
                .iter()
                .skip(self.skip(area.height))
                .map(|l| Line::from(l.clone()))
                .collect()
        };
        f.render_widget(
            Paragraph::new(lines).block(
                Block::bordered()
                    .title(format!(" {title} "))
                    .border_style(border),
            ),
            area,
        );
    }

    /// Where the terminal cursor goes in a box drawn in `area`.
    pub(crate) fn cursor_position(&self, area: Rect) -> Position {
        let row = self.row.saturating_sub(self.skip(area.height));
        let to_u16 = |n: usize| u16::try_from(n).unwrap_or(u16::MAX);
        Position {
            x: area
                .x
                .saturating_add(1)
                .saturating_add(to_u16(self.column))
                .min(area.right().saturating_sub(2)),
            y: area
                .y
                .saturating_add(1)
                .saturating_add(to_u16(row))
                .min(area.bottom().saturating_sub(2)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn typed(text: &str) -> TextArea {
        let mut area = TextArea::default();
        for c in text.chars() {
            area.handle_key(if c == '\n' {
                KeyCode::Enter
            } else {
                KeyCode::Char(c)
            });
        }
        area
    }

    #[test]
    fn enter_splits_lines_and_backspace_joins_them() {
        let mut area = typed("1 Sol Ring\n1 Opt");
        assert_eq!(area.text(), "1 Sol Ring\n1 Opt");
        assert_eq!(area.cursor(), (1, 5));
        for _ in 0..5 {
            area.handle_key(KeyCode::Left);
        }
        area.handle_key(KeyCode::Backspace);
        assert_eq!(area.text(), "1 Sol Ring1 Opt");
        assert_eq!(area.cursor(), (0, 10));
        area.handle_key(KeyCode::Enter);
        area.handle_key(KeyCode::Up);
        area.handle_key(KeyCode::End);
        area.handle_key(KeyCode::Delete);
        assert_eq!(area.text(), "1 Sol Ring1 Opt");
    }

    #[test]
    fn the_cursor_stays_inside_the_text() {
        let mut area = typed("ab\nc");
        area.handle_key(KeyCode::Up);
        area.handle_key(KeyCode::End);
        area.handle_key(KeyCode::Down);
        assert_eq!(area.cursor(), (1, 1), "clamped to the shorter line");
        for _ in 0..5 {
            area.handle_key(KeyCode::Down);
            area.handle_key(KeyCode::Right);
        }
        assert_eq!(area.cursor(), (1, 1));
        area.handle_key(KeyCode::Home);
        area.handle_key(KeyCode::Left);
        assert_eq!(area.cursor(), (0, 2), "left from a line's start goes up");
        assert!(!area.handle_key(KeyCode::Tab));
    }

    #[test]
    fn blank_text_shows_the_placeholder() {
        let area = TextArea::default().with_placeholder(&["1 Sol Ring"]);
        assert!(area.is_blank());
        assert!(!typed("x").is_blank());
        assert!(typed("  \n ").is_blank());
        let mut terminal =
            ratatui::Terminal::new(ratatui::backend::TestBackend::new(20, 4)).unwrap();
        terminal
            .draw(|f| area.draw(f, &Palette::terminal(), f.area(), "Paste", true))
            .unwrap();
        let text: String = terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(ratatui::buffer::Cell::symbol)
            .collect();
        assert!(text.contains("1 Sol Ring"));
    }
}
