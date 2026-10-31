//! A one-line text box: a cursor that moves, optional masking for passwords, a placeholder while empty, and an error line under it.

use crossterm::event::KeyCode;
use ratatui::Frame;
use ratatui::layout::{Position, Rect};
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::widgets::{Block, Paragraph};

use super::measure::BORDERS;
use crate::domain::theme::Palette;

/// Stands in for each typed character in a masked box.
pub(crate) const MASK: char = '•';

/// The box and its border take this many rows; an error adds one.
pub(crate) const BOX_HEIGHT: u16 = 3;

/// A one-line text box's state.
///
/// The cursor counts characters, not bytes, and always sits between 0 and the length of the value.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(crate) struct TextInput {
    value: String,
    /// Characters before the cursor.
    cursor: usize,
    masked: bool,
    placeholder: String,
    error: Option<String>,
}

impl TextInput {
    /// An empty box that shows each character as [`MASK`].
    pub(crate) fn masked() -> Self {
        Self {
            masked: true,
            ..Self::default()
        }
    }

    /// The box holding `value`, the cursor at its end.
    pub(crate) fn with_value(mut self, value: &str) -> Self {
        self.set_value(value);
        self
    }

    /// Text shown muted while the box is empty.
    pub(crate) fn with_placeholder(mut self, placeholder: &str) -> Self {
        placeholder.clone_into(&mut self.placeholder);
        self
    }

    /// Masks or shows what is typed, as a password's Show and Hide do.
    pub(crate) fn set_masked(&mut self, masked: bool) {
        self.masked = masked;
    }

    pub(crate) fn is_masked(&self) -> bool {
        self.masked
    }

    pub(crate) fn value(&self) -> &str {
        &self.value
    }

    /// Replaces the value and puts the cursor at its end.
    pub(crate) fn set_value(&mut self, value: &str) {
        value.clone_into(&mut self.value);
        self.cursor = self.len();
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.value.is_empty()
    }

    /// The value's length in characters.
    pub(crate) fn len(&self) -> usize {
        self.value.chars().count()
    }

    /// Characters before the cursor.
    pub(crate) fn cursor(&self) -> usize {
        self.cursor
    }

    pub(crate) fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }

    pub(crate) fn set_error(&mut self, error: Option<String>) {
        self.error = error;
    }

    /// The byte offset of the character at `chars`, or the end.
    fn byte_at(&self, chars: usize) -> usize {
        self.value
            .char_indices()
            .nth(chars)
            .map_or(self.value.len(), |(i, _)| i)
    }

    pub(crate) fn insert(&mut self, c: char) {
        let at = self.byte_at(self.cursor);
        self.value.insert(at, c);
        self.cursor = self.cursor.saturating_add(1);
    }

    /// Deletes the character before the cursor.
    pub(crate) fn backspace(&mut self) {
        let Some(before) = self.cursor.checked_sub(1) else {
            return;
        };
        let at = self.byte_at(before);
        self.value.remove(at);
        self.cursor = before;
    }

    /// Deletes the character under the cursor.
    pub(crate) fn delete(&mut self) {
        if self.cursor < self.len() {
            let at = self.byte_at(self.cursor);
            self.value.remove(at);
        }
    }

    pub(crate) fn left(&mut self) {
        self.cursor = self.cursor.saturating_sub(1);
    }

    pub(crate) fn right(&mut self) {
        self.cursor = self.cursor.saturating_add(1).min(self.len());
    }

    pub(crate) fn home(&mut self) {
        self.cursor = 0;
    }

    pub(crate) fn end(&mut self) {
        self.cursor = self.len();
    }

    /// Edits for typing, Backspace, Delete, Left, Right, Home and End. Returns whether the key was one of them; editing clears the error.
    pub(crate) fn handle_key(&mut self, code: KeyCode) -> bool {
        match code {
            KeyCode::Char(c) => self.insert(c),
            KeyCode::Backspace => self.backspace(),
            KeyCode::Delete => self.delete(),
            KeyCode::Left => self.left(),
            KeyCode::Right => self.right(),
            KeyCode::Home => self.home(),
            KeyCode::End => self.end(),
            _ => return false,
        }
        if matches!(
            code,
            KeyCode::Char(_) | KeyCode::Backspace | KeyCode::Delete
        ) {
            self.error = None;
        }
        true
    }

    /// The value as it shows: itself, or one [`MASK`] per character.
    pub(crate) fn shown(&self) -> String {
        if self.masked {
            MASK.to_string().repeat(self.len())
        } else {
            self.value.clone()
        }
    }

    /// Rows the box needs: the bordered box, and its error line when it has one.
    pub(crate) fn height(&self) -> u16 {
        BOX_HEIGHT.saturating_add(u16::from(self.error.is_some()))
    }

    /// Characters scrolled off the left of a box `width` cells wide, so the cursor stays in view.
    fn skip(&self, width: u16) -> usize {
        let inner = usize::from(width.saturating_sub(BORDERS));
        self.cursor.saturating_sub(inner.saturating_sub(1))
    }

    /// Draws the box titled `title` at the top of `area`, its border in the focus color while `focused`, and the error under it.
    pub(crate) fn draw(
        &self,
        f: &mut Frame<'_>,
        palette: &Palette,
        area: Rect,
        title: &str,
        focused: bool,
    ) {
        let box_area = Rect {
            height: BOX_HEIGHT.min(area.height),
            ..area
        };
        let border = if focused { palette.add } else { palette.border };
        let text = if self.value.is_empty() {
            Line::from(self.placeholder.clone()).style(palette.muted)
        } else {
            let skip = self.skip(box_area.width);
            Line::from(self.shown().chars().skip(skip).collect::<String>()).style(Style::new())
        };
        let mut block = Block::bordered().border_style(border);
        if !title.is_empty() {
            block = block.title(format!(" {title} "));
        }
        f.render_widget(Paragraph::new(text).block(block), box_area);
        if let Some(error) = &self.error
            && area.height > BOX_HEIGHT
        {
            let error_area = Rect {
                y: area.y.saturating_add(BOX_HEIGHT),
                height: 1,
                ..area
            };
            f.render_widget(Line::from(error.clone()).style(palette.error), error_area);
        }
    }

    /// Where the terminal cursor goes for a box drawn at the top of `area`. The caller sets it on the focused box, unless it is busy.
    pub(crate) fn cursor_position(&self, area: Rect) -> Position {
        let column =
            u16::try_from(self.cursor.saturating_sub(self.skip(area.width))).unwrap_or(u16::MAX);
        Position {
            x: area
                .x
                .saturating_add(1)
                .saturating_add(column)
                .min(area.right().saturating_sub(BORDERS)),
            y: area.y.saturating_add(1),
        }
    }
}

#[cfg(test)]
mod tests {
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    use super::*;

    fn typed(text: &str) -> TextInput {
        let mut input = TextInput::default();
        for c in text.chars() {
            input.handle_key(KeyCode::Char(c));
        }
        input
    }

    #[test]
    fn the_cursor_moves_and_edits_where_it_is() {
        let mut input = typed("héllo");
        assert_eq!(input.cursor(), 5);
        input.handle_key(KeyCode::Home);
        input.handle_key(KeyCode::Right);
        input.handle_key(KeyCode::Delete);
        assert_eq!(input.value(), "hllo", "Delete takes the é under the cursor");
        input.handle_key(KeyCode::Char('e'));
        input.handle_key(KeyCode::End);
        input.handle_key(KeyCode::Backspace);
        assert_eq!(input.value(), "hell");
        input.handle_key(KeyCode::Left);
        input.handle_key(KeyCode::Backspace);
        assert_eq!(input.value(), "hel");
        assert_eq!(input.cursor(), 2);
    }

    #[test]
    fn the_cursor_stays_in_range() {
        let mut input = typed("ab");
        for _ in 0..5 {
            input.right();
        }
        assert_eq!(input.cursor(), 2);
        input.home();
        input.backspace();
        input.left();
        assert_eq!((input.value(), input.cursor()), ("ab", 0));
        input.end();
        input.delete();
        assert_eq!(input.value(), "ab");
        assert!(!input.handle_key(KeyCode::Tab), "Tab is the form's");
    }

    #[test]
    fn typing_clears_the_error_and_masking_hides_the_value() {
        assert!(TextInput::masked().is_empty());
        let mut input = TextInput::masked().with_value("hunter2");
        input.set_error(Some("too short".to_owned()));
        assert_eq!(input.height(), BOX_HEIGHT + 1);
        assert_eq!(input.shown(), "•••••••");
        input.handle_key(KeyCode::Left);
        assert!(input.error().is_some(), "moving keeps the error");
        input.handle_key(KeyCode::Char('!'));
        assert!(input.error().is_none());
    }

    #[test]
    fn the_cursor_follows_the_typing_and_a_long_value_scrolls() {
        let area = Rect::new(0, 0, 8, 3);
        let mut input = typed("abc");
        assert_eq!(input.cursor_position(area), Position { x: 4, y: 1 });
        input.home();
        assert_eq!(input.cursor_position(area).x, 1);
        input.set_value("a value too long for the box");
        assert_eq!(
            input.cursor_position(area).x,
            6,
            "the last cell inside the border"
        );
    }

    #[test]
    fn the_box_shows_the_placeholder_until_typed_and_the_error_under_it() {
        let palette = Palette::terminal();
        let draw = |input: &TextInput| -> String {
            let mut terminal = Terminal::new(TestBackend::new(30, 4)).unwrap();
            terminal
                .draw(|f| input.draw(f, &palette, f.area(), "Email", true))
                .unwrap();
            terminal
                .backend()
                .buffer()
                .content()
                .iter()
                .map(ratatui::buffer::Cell::symbol)
                .collect()
        };
        let mut input = TextInput::default().with_placeholder("you@example.com");
        assert!(draw(&input).contains("you@example.com"));
        input.set_value("scotty@zwipe.net");
        input.set_error(Some("Taken".to_owned()));
        let screen = draw(&input);
        assert!(screen.contains("Email"));
        assert!(screen.contains("scotty@zwipe.net"));
        assert!(!screen.contains("you@example.com"));
        assert!(screen.contains("Taken"));
    }
}
