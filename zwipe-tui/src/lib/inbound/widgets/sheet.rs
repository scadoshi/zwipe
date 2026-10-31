//! The app's bottom sheet: a titled panel over the lower part of the screen, with a body that scrolls and footer buttons on its bottom edge.

use crossterm::event::KeyCode;
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Stylize;
use ratatui::text::Line;
use ratatui::widgets::{Block, Clear, Padding, Paragraph};

use super::action_bar;
use super::measure::{BORDERS, height_of};
use crate::domain::theme::Palette;

/// A sheet takes at most this share of the screen's height, in percent.
const MAX_HEIGHT_PERCENT: u16 = 70;

/// What a key did to the sheet.
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "for a sheet whose body only scrolls; the sheets so far handle their own keys"
    )
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SheetKey {
    /// The key scrolled the body, or did nothing.
    Stay,
    /// Esc or q closed the sheet.
    Close,
}

/// A bottom sheet: its title, body lines, footer keys, and how far the body is scrolled.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Sheet {
    pub(crate) title: String,
    pub(crate) body: Vec<Line<'static>>,
    /// Each footer button as a key and what it does.
    pub(crate) footer: Vec<(String, String)>,
    /// The first body line shown.
    pub(crate) scroll: usize,
    /// A body line kept in view, such as the selected row of a list.
    pub(crate) focus: Option<usize>,
    /// Whether the header carries a `?`.
    pub(crate) hint: bool,
}

impl Sheet {
    pub(crate) fn new(title: &str, body: Vec<Line<'static>>) -> Self {
        Self {
            title: title.to_owned(),
            body,
            footer: Vec::new(),
            scroll: 0,
            focus: None,
            hint: false,
        }
    }

    /// Keeps body line `line` in view, scrolling the least it takes.
    pub(crate) fn with_focus(mut self, line: usize) -> Self {
        self.focus = Some(line);
        self
    }

    /// Puts a `?` in the header's right corner, for a sheet with a hint to reopen.
    pub(crate) fn with_hint(mut self) -> Self {
        self.hint = true;
        self
    }

    pub(crate) fn with_footer(mut self, key: &str, action: &str) -> Self {
        self.footer.push((key.to_owned(), action.to_owned()));
        self
    }

    /// Scrolls down a line, stopping at the last body line.
    #[cfg_attr(
        not(test),
        expect(dead_code, reason = "for a sheet whose body only scrolls")
    )]
    pub(crate) fn scroll_down(&mut self) {
        self.scroll = self
            .scroll
            .saturating_add(1)
            .min(self.body.len().saturating_sub(1));
    }

    #[cfg_attr(
        not(test),
        expect(dead_code, reason = "for a sheet whose body only scrolls")
    )]
    pub(crate) fn scroll_up(&mut self) {
        self.scroll = self.scroll.saturating_sub(1);
    }

    /// Scrolls with j/k, the arrows and Page Up/Down; closes on Esc or q.
    #[cfg_attr(
        not(test),
        expect(dead_code, reason = "for a sheet whose body only scrolls")
    )]
    pub(crate) fn handle_key(&mut self, code: KeyCode) -> SheetKey {
        match code {
            KeyCode::Esc | KeyCode::Char('q') => return SheetKey::Close,
            KeyCode::Down | KeyCode::Char('j') => self.scroll_down(),
            KeyCode::Up | KeyCode::Char('k') => self.scroll_up(),
            KeyCode::PageDown => {
                for _ in 0..10 {
                    self.scroll_down();
                }
            }
            KeyCode::PageUp => self.scroll = self.scroll.saturating_sub(10),
            KeyCode::Home => self.scroll = 0,
            _ => {}
        }
        SheetKey::Stay
    }

    /// The body lines from the scroll position, never past the point where the last line reaches the bottom, and moved as little as it takes to show the focused line.
    fn visible(&self, rows: usize) -> &[Line<'static>] {
        let mut start = self.scroll.min(self.body.len().saturating_sub(rows));
        if let Some(focus) = self.focus {
            if focus < start {
                start = focus;
            } else if focus >= start.saturating_add(rows) {
                start = focus.saturating_add(1).saturating_sub(rows);
            }
        }
        self.body.get(start..).unwrap_or_default()
    }

    /// Draws the sheet over the bottom of `area`, as tall as its body up to most of the screen.
    pub(crate) fn draw(&self, f: &mut Frame<'_>, palette: &Palette, area: Rect) {
        let footer: Vec<(&str, &str)> = self
            .footer
            .iter()
            .map(|(k, a)| (k.as_str(), a.as_str()))
            .collect();
        let inner = frame(
            f,
            palette,
            area,
            &self.title,
            height_of(self.body.len()),
            self.hint,
            &footer,
        );
        let rows = usize::from(inner.height);
        f.render_widget(Paragraph::new(self.visible(rows).to_vec()), inner);
    }
}

/// Clears and draws a bottom sheet's frame over the bottom of `area`: the title centered in the third accent, a `?` in its right corner when `hint`, and the footer keys on its bottom edge, tall enough for `body` rows up to most of the screen. Returns the area inside, for a body the caller draws, such as a form.
pub(crate) fn frame(
    f: &mut Frame<'_>,
    palette: &Palette,
    area: Rect,
    title: &str,
    body: u16,
    hint: bool,
    footer: &[(&str, &str)],
) -> Rect {
    let max = area.height.saturating_mul(MAX_HEIGHT_PERCENT) / 100;
    let height = body
        .saturating_add(BORDERS)
        .min(max.max(BORDERS.saturating_add(1)))
        .min(area.height);
    let [_, sheet] =
        Layout::vertical([Constraint::Fill(1), Constraint::Length(height)]).areas(area);
    let mut block = Block::bordered()
        .title(
            Line::from(format!(" {title} "))
                .style(palette.tertiary)
                .bold()
                .centered(),
        )
        .title_bottom(action_bar::line(palette, footer))
        .border_style(palette.accent)
        .style(palette.base)
        .padding(Padding::horizontal(1));
    if hint {
        block = block.title(Line::from(" ? ").style(palette.muted).right_aligned());
    }
    let inner = block.inner(sheet);
    f.render_widget(Clear, sheet);
    f.render_widget(block, sheet);
    inner
}

#[cfg(test)]
mod tests {
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    use super::*;

    fn numbered(count: usize) -> Sheet {
        Sheet::new(
            "Rules",
            (1..=count)
                .map(|i| Line::from(format!("line {i}")))
                .collect(),
        )
        .with_footer("Esc", "close")
    }

    fn text(sheet: &Sheet, width: u16, height: u16) -> String {
        let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
        terminal
            .draw(|f| sheet.draw(f, &Palette::terminal(), f.area()))
            .unwrap();
        terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(ratatui::buffer::Cell::symbol)
            .collect()
    }

    #[test]
    fn scrolling_stops_at_both_ends_and_esc_closes() {
        let mut sheet = numbered(3);
        sheet.scroll_up();
        assert_eq!(sheet.scroll, 0);
        for _ in 0..5 {
            assert_eq!(sheet.handle_key(KeyCode::Char('j')), SheetKey::Stay);
        }
        assert_eq!(sheet.scroll, 2);
        assert_eq!(sheet.handle_key(KeyCode::Esc), SheetKey::Close);
    }

    #[test]
    fn the_sheet_sits_at_the_bottom_with_its_title_and_footer() {
        let screen = text(&numbered(2), 30, 10);
        assert!(screen.contains("Rules"));
        assert!(screen.contains("line 1"));
        assert!(screen.contains("Esc close"));
        assert!(
            !screen.chars().take(30 * 6).any(|c| c != ' '),
            "the top of the screen stays clear"
        );
    }

    #[test]
    fn the_focused_line_stays_in_view_and_the_hint_corner_shows() {
        let sheet = numbered(40).with_focus(30).with_hint();
        let screen = text(&sheet, 30, 10);
        assert!(screen.contains("line 31"), "scrolled to the focus");
        assert!(!screen.contains("line 1 "));
        assert!(screen.contains(" ? "));
    }

    #[test]
    fn a_long_body_scrolls_but_keeps_the_screen_full() {
        let mut sheet = numbered(40);
        assert!(!text(&sheet, 30, 10).contains("line 8"));
        sheet.scroll = 39;
        let screen = text(&sheet, 30, 10);
        assert!(screen.contains("line 40"));
        assert!(screen.contains("line 36"), "the last lines fill the sheet");
    }
}
