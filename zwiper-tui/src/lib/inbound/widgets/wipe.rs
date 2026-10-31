//! The theme wipe, after orangui's `use_theme_follow`: a new theme sweeps over the old one in 0.4 s, top to bottom, or bottom to top when going from light to dark.
//!
//! The screen is drawn in the new theme; the rows the sweep has not reached yet are recolored back to the old theme, role for role.

use std::time::{Duration, Instant};

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Color;

use crate::domain::theme::Palette;

/// How long a sweep takes.
pub(crate) const DURATION: Duration = Duration::from_millis(400);

/// A sweep from one theme to the next.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Wipe {
    from: Palette,
    started: Instant,
    /// Light to dark sweeps from the bottom up.
    upward: bool,
}

impl Wipe {
    /// A sweep starting at `now` from `from` to `to`.
    pub(crate) fn new(from: &Palette, to: &Palette, now: Instant) -> Self {
        Self {
            from: *from,
            started: now,
            upward: !from.is_dark() && to.is_dark(),
        }
    }

    /// Whether the sweep is over at `now`.
    pub(crate) fn done(&self, now: Instant) -> bool {
        now.saturating_duration_since(self.started) >= DURATION
    }

    /// How far the sweep has gone at `now`, from 0 to 1, on the app's `cubic-bezier(0.65, 0, 0.35, 1)`, close to an ease-in-out cubic.
    fn progress(&self, now: Instant) -> f64 {
        let t = (now.saturating_duration_since(self.started).as_secs_f64()
            / DURATION.as_secs_f64())
        .clamp(0.0, 1.0);
        if t < 0.5 {
            4.0 * t * t * t
        } else {
            1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
        }
    }

    /// Recolors the rows of `area` the sweep has not reached at `now` from `to` back to the old theme.
    pub(crate) fn apply(&self, buffer: &mut Buffer, area: Rect, to: &Palette, now: Instant) {
        let pairs: Vec<(Color, Color)> = to
            .styles()
            .iter()
            .zip(self.from.styles())
            .flat_map(|(new, old)| [(new.fg, old.fg), (new.bg, old.bg)])
            .filter_map(|(new, old)| new.zip(old))
            .collect();
        let back = |color: Color| {
            pairs
                .iter()
                .find(|(new, _)| *new == color)
                .map_or(color, |(_, old)| *old)
        };
        #[expect(
            clippy::as_conversions,
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "a share of a terminal's height fits u16"
        )]
        let swept = (self.progress(now) * f64::from(area.height)).round() as u16;
        let rows = if self.upward {
            area.top()..area.bottom().saturating_sub(swept)
        } else {
            area.top().saturating_add(swept)..area.bottom()
        };
        for y in rows {
            for x in area.left()..area.right() {
                if let Some(cell) = buffer.cell_mut((x, y)) {
                    let (fg, bg) = (back(cell.fg), back(cell.bg));
                    cell.set_fg(fg);
                    cell.set_bg(bg);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use ratatui::widgets::{Block, Widget};

    use super::*;

    fn drawn(palette: &Palette) -> Buffer {
        let area = Rect::new(0, 0, 4, 10);
        let mut buffer = Buffer::empty(area);
        Block::new().style(palette.base).render(area, &mut buffer);
        buffer
    }

    #[test]
    fn the_new_theme_sweeps_down_over_the_old_one() {
        let old = Palette::for_account("gruvbox", true);
        let new = Palette::for_account("nord", true);
        let now = Instant::now();
        let wipe = Wipe::new(&old, &new, now);
        let mut buffer = drawn(&new);
        let area = buffer.area;
        wipe.apply(&mut buffer, area, &new, now + DURATION / 2);
        let bg = |y: u16| buffer.cell((0, y)).unwrap().bg;
        assert_eq!(bg(0), new.base.bg.unwrap(), "the top is swept");
        assert_eq!(bg(9), old.base.bg.unwrap(), "the bottom is not yet");
        assert!(wipe.done(now + DURATION));
    }

    #[test]
    fn light_to_dark_sweeps_up() {
        let old = Palette::for_account("gruvbox", false);
        let new = Palette::for_account("gruvbox", true);
        let now = Instant::now();
        let wipe = Wipe::new(&old, &new, now);
        let mut buffer = drawn(&new);
        let area = buffer.area;
        wipe.apply(&mut buffer, area, &new, now + DURATION / 4);
        assert_eq!(buffer.cell((0, 0)).unwrap().bg, old.base.bg.unwrap());
        assert_eq!(buffer.cell((0, 9)).unwrap().bg, new.base.bg.unwrap());
    }
}
