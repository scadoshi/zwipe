//! Toasts: short success, error, info and warning messages stacked in the top-left corner over the header, each gone when its time is up.
//!
//! The app's three duration bands are [`QUICK`] for routine confirmations, [`NORMAL`] for a result worth reading (the default) and [`LONG`] for failures and anything naming a next step. At most [`MAX_TOASTS`] are up; the newest sits on top with the older ones tucked behind it as one-row slivers, and Ctrl+T expands the stack so each shows in full.
//!
//! The loop prunes expired toasts on every pass and keeps redrawing on its animation tick while any is up, so they leave on time without a key press.

use std::time::{Duration, Instant};

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Clear, Padding, Paragraph, Wrap};

use super::measure::{BORDERS, height_of, width_of};
use crate::domain::theme::Palette;

/// Routine confirmations: saved, copied, added.
pub(crate) const QUICK: Duration = Duration::from_secs(3);

/// A result worth reading, and the default.
pub(crate) const NORMAL: Duration = Duration::from_secs(5);

/// Failures, and anything that names a next step.
pub(crate) const LONG: Duration = Duration::from_secs(8);

/// How long a toast raised without a duration stays up.
pub(crate) const LIFETIME: Duration = NORMAL;

/// At most this many toasts are up; a newer one pushes out the oldest.
pub(crate) const MAX_TOASTS: usize = 3;

/// A toast is at most this wide, border included, in cells: the app's `16rem`.
const MAX_WIDTH: u16 = 34;

/// A collapsed toast shows at most this many lines of its message.
const COLLAPSED_LINES: usize = 2;

/// How long a new toast takes to drop into place, from the app's `toast-drop-in`.
const DROP_IN: Duration = Duration::from_millis(220);

/// What a toast reports, which sets its border color.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ToastKind {
    Success,
    Error,
    Info,
    #[cfg_attr(
        not(test),
        expect(dead_code, reason = "the app raises no warning toast on these screens")
    )]
    Warning,
}

impl ToastKind {
    fn border(self, palette: &Palette) -> Style {
        match self {
            Self::Success => palette.border_success,
            Self::Error => palette.border_error,
            Self::Warning => palette.border_warning,
            Self::Info => palette.border,
        }
    }
}

/// One message, when it arrived and when it leaves.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Toast {
    pub(crate) kind: ToastKind,
    pub(crate) message: String,
    pub(crate) shown_at: Instant,
    pub(crate) expires_at: Instant,
}

/// The toasts that are up, oldest first, and whether the stack is expanded.
#[derive(Debug, Default)]
pub(crate) struct Toasts {
    items: Vec<Toast>,
    expanded: bool,
}

impl Toasts {
    /// Shows `message` from now for [`NORMAL`].
    pub(crate) fn push(&mut self, kind: ToastKind, message: String) {
        self.push_for(kind, message, LIFETIME);
    }

    /// Shows `message` from now for `duration`, one of [`QUICK`], [`NORMAL`] or [`LONG`].
    pub(crate) fn push_for(&mut self, kind: ToastKind, message: String, duration: Duration) {
        self.push_for_at(kind, message, duration, Instant::now());
    }

    /// Shows `message` from `now` for [`NORMAL`].
    pub(crate) fn push_at(&mut self, kind: ToastKind, message: String, now: Instant) {
        self.push_for_at(kind, message, LIFETIME, now);
    }

    /// Shows `message` from `now` for `duration`, dropping the oldest toast past [`MAX_TOASTS`].
    pub(crate) fn push_for_at(
        &mut self,
        kind: ToastKind,
        message: String,
        duration: Duration,
        now: Instant,
    ) {
        let expires_at = now.checked_add(duration).unwrap_or(now);
        self.items.push(Toast {
            kind,
            message,
            shown_at: now,
            expires_at,
        });
        let extra = self.items.len().saturating_sub(MAX_TOASTS);
        self.items.drain(..extra);
    }

    /// Drops the toasts whose time is up at `now`. The stack collapses again once it is empty.
    pub(crate) fn prune(&mut self, now: Instant) {
        self.items.retain(|toast| toast.expires_at > now);
        if self.items.is_empty() {
            self.expanded = false;
        }
    }

    /// Expands the stack so every message shows in full, or collapses it again. Does nothing while no toast is up.
    pub(crate) fn toggle_expanded(&mut self) {
        self.expanded = !self.expanded && !self.items.is_empty();
    }

    pub(crate) fn is_expanded(&self) -> bool {
        self.expanded
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

    /// Draws the stack in the top-left corner of `area`, newest on top.
    pub(crate) fn draw(&self, f: &mut Frame<'_>, palette: &Palette, area: Rect) {
        self.draw_at(f, palette, area, Instant::now());
    }

    /// [`Toasts::draw`] as of `now`, which places a toast still dropping in.
    pub(crate) fn draw_at(&self, f: &mut Frame<'_>, palette: &Palette, area: Rect, now: Instant) {
        let left = area.x.saturating_add(1).min(area.right());
        let room = area.width.saturating_sub(2).max(area.width.min(BORDERS));
        let newest_first: Vec<&Toast> = self.items.iter().rev().collect();
        if self.expanded {
            let mut top = area.y;
            for toast in newest_first {
                let Some(rect) = place(toast, left, top, room, None, area) else {
                    break;
                };
                render(f, palette, toast, rect, None, now);
                top = rect.bottom();
            }
            return;
        }
        let Some((newest, older)) = newest_first.split_first() else {
            return;
        };
        let Some(front) = place(newest, left, area.y, room, Some(COLLAPSED_LINES), area) else {
            return;
        };
        // Older toasts tuck behind, each showing one more row of its bottom edge.
        for (depth, toast) in older.iter().enumerate().rev() {
            let sliver = u16::try_from(depth).unwrap_or(u16::MAX).saturating_add(1);
            let bottom = front.bottom().saturating_add(sliver);
            let Some(mut rect) = place(toast, left, area.y, room, Some(COLLAPSED_LINES), area)
            else {
                continue;
            };
            rect.y = bottom.saturating_sub(rect.height).max(area.y);
            if rect.bottom() > area.bottom() {
                continue;
            }
            render(f, palette, toast, rect, Some(COLLAPSED_LINES), now);
        }
        render(f, palette, newest, front, Some(COLLAPSED_LINES), now);
    }
}

/// Where `toast` goes with its top at `top`, as wide as its message up to the cap and as tall as its wrapped lines, at most `lines` of them. `None` when it does not fit.
fn place(
    toast: &Toast,
    left: u16,
    top: u16,
    room: u16,
    lines: Option<usize>,
    area: Rect,
) -> Option<Rect> {
    let width = width_of(&toast.message)
        .saturating_add(BORDERS)
        .saturating_add(BORDERS)
        .min(MAX_WIDTH)
        .min(room);
    let inner = width.saturating_sub(BORDERS).saturating_sub(BORDERS);
    let wrapped = body(toast, Style::new()).line_count(inner);
    let shown = lines.map_or(wrapped, |cap| wrapped.min(cap));
    let height = height_of(shown).saturating_add(BORDERS);
    let rect = Rect {
        x: left,
        y: top,
        width,
        height,
    };
    (width > BORDERS && rect.bottom() <= area.bottom()).then_some(rect)
}

fn body(toast: &Toast, style: Style) -> Paragraph<'static> {
    Paragraph::new(Line::from(toast.message.clone()).style(style).centered())
        .wrap(Wrap { trim: true })
}

/// Draws one toast in `rect`, its message clamped to `lines` with an ellipsis when it runs over. A toast still dropping in at `now` draws faint, as the app's fades in.
fn render(
    f: &mut Frame<'_>,
    palette: &Palette,
    toast: &Toast,
    rect: Rect,
    lines: Option<usize>,
    now: Instant,
) {
    let inner = rect.width.saturating_sub(BORDERS).saturating_sub(BORDERS);
    let arriving = now.saturating_duration_since(toast.shown_at) < DROP_IN;
    let base = if arriving {
        palette.base.add_modifier(Modifier::DIM)
    } else {
        palette.base
    };
    let full = body(toast, base);
    let clamped = lines.is_some_and(|cap| full.line_count(inner) > cap);
    let paragraph = if clamped {
        Paragraph::new(Line::from(clamp(&toast.message, inner, lines.unwrap_or(1))).centered())
            .style(base)
            .wrap(Wrap { trim: true })
    } else {
        full
    };
    f.render_widget(Clear, rect);
    f.render_widget(
        paragraph.block(
            Block::bordered()
                .border_style(toast.kind.border(palette))
                .style(base)
                .padding(Padding::horizontal(1)),
        ),
        rect,
    );
}

/// `message` cut to fit `lines` lines of `width` cells, ending in an ellipsis.
fn clamp(message: &str, width: u16, lines: usize) -> String {
    let budget = usize::from(width)
        .saturating_mul(lines)
        .saturating_sub(lines.saturating_mul(2));
    let mut out: String = message.chars().take(budget.saturating_sub(1)).collect();
    out.push('…');
    out
}

#[cfg(test)]
mod tests {
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    use super::*;

    fn rows(toasts: &Toasts, width: u16, height: u16, now: Instant) -> Vec<String> {
        let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
        terminal
            .draw(|f| toasts.draw_at(f, &Palette::terminal(), f.area(), now))
            .unwrap();
        let buffer = terminal.backend().buffer();
        (0..height)
            .map(|y| {
                (0..width)
                    .map(|x| buffer.cell((x, y)).unwrap().symbol())
                    .collect()
            })
            .collect()
    }

    #[test]
    fn each_band_sets_how_long_a_toast_stays() {
        let now = Instant::now();
        let mut toasts = Toasts::default();
        toasts.push_for_at(ToastKind::Success, "Saved".to_owned(), QUICK, now);
        toasts.push_at(ToastKind::Info, "Read me".to_owned(), now);
        toasts.push_for_at(ToastKind::Error, "Offline".to_owned(), LONG, now);
        toasts.prune(now + QUICK);
        assert_eq!(toasts.len(), 2, "the quick one has gone");
        toasts.prune(now + NORMAL);
        assert_eq!(toasts.items()[0].message, "Offline");
        toasts.prune(now + LONG);
        assert!(toasts.is_empty());
        assert_eq!(
            (QUICK, NORMAL, LONG, LIFETIME),
            (QUICK, NORMAL, LONG, NORMAL)
        );
    }

    #[test]
    fn a_fourth_toast_pushes_out_the_oldest() {
        let mut toasts = Toasts::default();
        for message in ["first", "second", "third", "fourth"] {
            toasts.push(ToastKind::Info, message.to_owned());
        }
        let messages: Vec<&str> = toasts.items().iter().map(|t| t.message.as_str()).collect();
        assert_eq!(messages, ["second", "third", "fourth"]);
    }

    #[test]
    fn the_newest_sits_top_left_with_the_older_ones_tucked_behind() {
        let now = Instant::now() - Duration::from_secs(1);
        let mut toasts = Toasts::default();
        for message in ["first", "second", "third"] {
            toasts.push_at(ToastKind::Info, message.to_owned(), now);
        }
        let screen = rows(&toasts, 40, 12, Instant::now());
        assert!(screen[1].contains("third"), "the newest on top: {screen:?}");
        assert!(screen[0].starts_with(" ┌"), "one cell in from the left");
        assert!(
            !screen.concat().contains("second"),
            "older ones show only an edge"
        );
        assert!(
            screen[3].contains('└') && screen[4].contains('└'),
            "{screen:?}"
        );
    }

    #[test]
    fn expanding_shows_every_message_and_collapses_when_they_leave() {
        let now = Instant::now() - Duration::from_secs(1);
        let mut toasts = Toasts::default();
        toasts.toggle_expanded();
        assert!(!toasts.is_expanded(), "nothing to expand");
        for message in ["first", "second"] {
            toasts.push_at(ToastKind::Success, message.to_owned(), now);
        }
        toasts.toggle_expanded();
        let screen = rows(&toasts, 40, 12, Instant::now()).concat();
        assert!(screen.contains("first") && screen.contains("second"));
        toasts.prune(now + LONG);
        assert!(!toasts.is_expanded());
    }

    #[test]
    fn a_long_message_is_clamped_to_two_lines_until_expanded() {
        let now = Instant::now() - Duration::from_secs(1);
        let mut toasts = Toasts::default();
        let long = "word ".repeat(40);
        toasts.push_at(ToastKind::Error, long, now);
        let collapsed = rows(&toasts, 60, 20, Instant::now());
        assert!(collapsed.concat().contains('…'));
        assert!(collapsed[4].trim().is_empty(), "two lines and a border");
        toasts.toggle_expanded();
        let expanded = rows(&toasts, 60, 20, Instant::now());
        assert_ne!(expanded[4].trim(), "");
    }

    #[test]
    fn a_toast_fits_a_tiny_terminal() {
        let mut toasts = Toasts::default();
        toasts.push(
            ToastKind::Error,
            "a long message that cannot fit".to_owned(),
        );
        rows(&toasts, 6, 2, Instant::now());
        rows(&toasts, 1, 1, Instant::now());
    }
}
