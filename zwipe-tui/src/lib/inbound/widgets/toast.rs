//! Toasts: short success, error and info messages stacked in the bottom-right corner, each gone after a few seconds.
//!
//! The loop prunes expired toasts on every pass and keeps redrawing on its animation tick while any is up, so they leave on time without a key press.

use std::time::{Duration, Instant};

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::widgets::{Block, Clear, Padding, Paragraph, Wrap};

use super::measure::{BORDERS, height_of, width_of};
use crate::domain::theme::Palette;

/// How long a toast stays up.
pub(crate) const LIFETIME: Duration = Duration::from_secs(3);

/// At most this many toasts show at once; older ones wait their turn below the fold.
const MAX_SHOWN: usize = 3;

/// A toast is at most this wide, border included, in cells.
const MAX_WIDTH: u16 = 44;

/// What a toast reports, which sets its color.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ToastKind {
    Success,
    Error,
    Info,
    Warning,
}

impl ToastKind {
    fn style(self, palette: &Palette) -> Style {
        match self {
            Self::Success => palette.add,
            Self::Error => palette.error,
            Self::Info => palette.accent,
            Self::Warning => palette.warning,
        }
    }
}

/// One message and when it leaves.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Toast {
    pub(crate) kind: ToastKind,
    pub(crate) message: String,
    pub(crate) expires_at: Instant,
}

/// The toasts that are up, oldest first.
#[derive(Debug, Default)]
pub(crate) struct Toasts {
    items: Vec<Toast>,
}

impl Toasts {
    /// Shows `message` from now for [`LIFETIME`].
    pub(crate) fn push(&mut self, kind: ToastKind, message: String) {
        self.push_at(kind, message, Instant::now());
    }

    /// Shows `message` from `now` for [`LIFETIME`].
    pub(crate) fn push_at(&mut self, kind: ToastKind, message: String, now: Instant) {
        let expires_at = now.checked_add(LIFETIME).unwrap_or(now);
        self.items.push(Toast {
            kind,
            message,
            expires_at,
        });
    }

    /// Drops the toasts whose time is up at `now`.
    pub(crate) fn prune(&mut self, now: Instant) {
        self.items.retain(|toast| toast.expires_at > now);
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub(crate) fn len(&self) -> usize {
        self.items.len()
    }

    /// The toasts that are up, oldest first.
    pub(crate) fn items(&self) -> &[Toast] {
        &self.items
    }

    /// Draws the newest few in the bottom-right corner of `area`, newest at the bottom.
    pub(crate) fn draw(&self, f: &mut Frame<'_>, palette: &Palette, area: Rect) {
        let newest = self.items.iter().rev().take(MAX_SHOWN);
        let mut bottom = area.bottom();
        for toast in newest {
            let width = width_of(&toast.message)
                .saturating_add(BORDERS)
                .saturating_add(BORDERS)
                .min(MAX_WIDTH)
                .min(area.width);
            let body = Paragraph::new(Line::from(toast.message.clone()).style(palette.base))
                .wrap(Wrap { trim: true });
            let inner_width = width.saturating_sub(BORDERS).saturating_sub(BORDERS);
            let height = height_of(body.line_count(inner_width))
                .saturating_add(BORDERS)
                .min(bottom.saturating_sub(area.y));
            if height <= BORDERS {
                break;
            }
            let rect = Rect {
                x: area.right().saturating_sub(width),
                y: bottom.saturating_sub(height),
                width,
                height,
            };
            f.render_widget(Clear, rect);
            f.render_widget(
                body.block(
                    Block::bordered()
                        .border_style(toast.kind.style(palette))
                        .style(palette.base)
                        .padding(Padding::horizontal(1)),
                ),
                rect,
            );
            bottom = rect.y;
        }
    }
}

#[cfg(test)]
mod tests {
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    use super::*;

    #[test]
    fn toasts_leave_after_their_lifetime() {
        let now = Instant::now();
        let mut toasts = Toasts::default();
        toasts.push_at(ToastKind::Success, "Saved".to_owned(), now);
        toasts.push_at(
            ToastKind::Error,
            "Offline".to_owned(),
            now + Duration::from_secs(1),
        );
        toasts.prune(now + LIFETIME - Duration::from_millis(1));
        assert_eq!(toasts.len(), 2);
        toasts.prune(now + LIFETIME);
        assert_eq!(toasts.items()[0].message, "Offline", "the first has gone");
        toasts.prune(now + LIFETIME + Duration::from_secs(1));
        assert!(toasts.is_empty());
    }

    #[test]
    fn toasts_stack_in_the_bottom_right_newest_lowest() {
        let mut toasts = Toasts::default();
        for message in ["first", "second", "third", "fourth"] {
            toasts.push(ToastKind::Info, message.to_owned());
        }
        let palette = Palette::terminal();
        let mut terminal = Terminal::new(TestBackend::new(40, 12)).unwrap();
        terminal
            .draw(|f| toasts.draw(f, &palette, f.area()))
            .unwrap();
        let buffer = terminal.backend().buffer();
        let row = |y: u16| -> String {
            (0..40)
                .map(|x| buffer.cell((x, y)).unwrap().symbol())
                .collect()
        };
        assert!(row(10).contains("fourth"), "the newest sits at the bottom");
        assert!(row(7).contains("third"));
        assert!(row(4).contains("second"));
        let shown: String = (0..12).map(row).collect();
        assert!(!shown.contains("first"), "only three show at once");
        assert!(
            row(10).trim_end().ends_with('│'),
            "flush with the right edge"
        );
    }

    #[test]
    fn a_toast_fits_a_tiny_terminal() {
        let mut toasts = Toasts::default();
        toasts.push(
            ToastKind::Error,
            "a long message that cannot fit".to_owned(),
        );
        let mut terminal = Terminal::new(TestBackend::new(6, 2)).unwrap();
        terminal
            .draw(|f| toasts.draw(f, &Palette::terminal(), f.area()))
            .unwrap();
    }
}
