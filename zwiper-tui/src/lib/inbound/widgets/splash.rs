//! The opening, ported from orangui's `Splash`: an overlay over the first screen while the backdrop rains in, the ZWIPE mark decodes over it and leaves the way it came, then the overlay dissolves.
//!
//! The timeline is orangui's at full pace: the mark mounts at 400 ms and decodes, starts to leave at 1350 ms over 450 ms, and at 1800 ms the overlay starts to fade, which opens the gate the first screen's entrances wait on. It is gone at 2100 ms. A terminal has no opacity, so the fade dissolves the overlay cell by cell. Any key ends the opening early.

use std::time::{Duration, Instant};

use ratatui::Frame;
use ratatui::layout::Constraint;
use ratatui::text::{Line, Text};
use ratatui::widgets::{Block, Clear, Paragraph};
use zwipe_core::domain::logo;

use super::{grid, measure};
use crate::domain::decode;
use crate::domain::theme::Palette;

/// When the mark mounts and starts to decode.
const MARK_AT: Duration = Duration::from_millis(400);

/// When the mark starts to leave.
const LEAVE_AT: Duration = Duration::from_millis(1350);

/// When the overlay starts to fade and the first screen's entrances start.
pub(crate) const OUT_AT: Duration = Duration::from_millis(1800);

/// How long the fade takes.
const FADE: Duration = Duration::from_millis(300);

/// The opening, from when it started.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Splash {
    pub(crate) started: Instant,
}

impl Splash {
    pub(crate) fn new(started: Instant) -> Self {
        Self { started }
    }

    /// When the gate opens: the first screen's entrances start here.
    pub(crate) fn gate(&self) -> Instant {
        self.started.checked_add(OUT_AT).unwrap_or(self.started)
    }

    /// Whether the overlay is gone at `now`.
    pub(crate) fn done(&self, now: Instant) -> bool {
        now.saturating_duration_since(self.started) >= OUT_AT.saturating_add(FADE)
    }

    /// The mark as it looks `elapsed` into the opening, or `None` before it mounts.
    fn mark(elapsed: Duration) -> Option<String> {
        if elapsed < MARK_AT {
            return None;
        }
        Some(if elapsed < LEAVE_AT {
            decode::at(logo::ZWIPE, elapsed.saturating_sub(MARK_AT))
        } else {
            decode::leaving(logo::ZWIPE, elapsed.saturating_sub(LEAVE_AT))
        })
    }

    /// Draws the overlay as of `now`.
    pub(crate) fn draw(&self, f: &mut Frame<'_>, palette: &Palette, now: Instant) {
        let elapsed = now.saturating_duration_since(self.started);
        if self.done(now) {
            return;
        }
        let area = f.area();
        let fading = elapsed.saturating_sub(OUT_AT);
        if elapsed < OUT_AT {
            f.render_widget(Clear, area);
            f.render_widget(Block::new().style(palette.base), area);
            grid::draw(f, palette, area, Some(elapsed));
            if let Some(mark) = Self::mark(elapsed) {
                let lines: Vec<Line<'static>> =
                    mark.lines().map(|l| Line::from(l.to_owned())).collect();
                let width = mark.lines().map(measure::width_of).max().unwrap_or(0);
                let spot = area.centered(
                    Constraint::Length(width),
                    Constraint::Length(measure::height_of(lines.len())),
                );
                f.render_widget(Paragraph::new(Text::from(lines).style(palette.logo)), spot);
            }
            return;
        }
        // The dissolve: each cell holds on until its own moment in the fade.
        let mut held = Vec::new();
        for y in area.top()..area.bottom() {
            for x in area.left()..area.right() {
                let moment = u64::from(x)
                    .wrapping_mul(2_654_435_761)
                    .wrapping_add(u64::from(y).wrapping_mul(40_503))
                    .checked_rem(u64::try_from(FADE.as_millis()).unwrap_or(1))
                    .unwrap_or(0);
                if u128::from(moment) >= fading.as_millis() {
                    held.push((x, y));
                }
            }
        }
        let (Some(bg), fg) = (palette.base.bg, palette.base.fg) else {
            return;
        };
        let buffer = f.buffer_mut();
        for (x, y) in held {
            if let Some(cell) = buffer.cell_mut((x, y)) {
                cell.reset();
                cell.set_bg(bg);
                if let Some(fg) = fg {
                    cell.set_fg(fg);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    use super::*;

    fn screen(splash: &Splash, at: Duration) -> String {
        let mut terminal = Terminal::new(TestBackend::new(60, 14)).unwrap();
        terminal
            .draw(|f| {
                f.render_widget(Paragraph::new("under the splash"), f.area());
                splash.draw(
                    f,
                    &Palette::for_account("gruvbox", true),
                    splash.started.checked_add(at).expect("the moment fits"),
                );
            })
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
    fn the_mark_decodes_then_leaves_then_the_overlay_dissolves() {
        let splash = Splash::new(Instant::now());
        let before = screen(&splash, Duration::from_millis(100));
        assert!(!before.contains("under"), "the overlay covers the screen");
        assert!(before.chars().all(|c| c == ' '), "no mark yet");
        let settled = screen(&splash, MARK_AT + decode::DURATION);
        assert!(settled.contains('█') || settled.chars().any(|c| c != ' '));
        let gone = screen(&splash, OUT_AT + FADE);
        assert!(gone.contains("under the splash"));
        assert!(splash.done(splash.started + OUT_AT + FADE));
        assert!(!splash.done(splash.started + OUT_AT));
        assert_eq!(splash.gate(), splash.started + OUT_AT);
    }

    #[test]
    fn the_splash_fits_a_tiny_terminal() {
        let splash = Splash::new(Instant::now());
        for ms in [0, 500, 1400, 1900] {
            let mut terminal = Terminal::new(TestBackend::new(3, 2)).unwrap();
            terminal
                .draw(|f| {
                    splash.draw(
                        f,
                        &Palette::terminal(),
                        splash.started + Duration::from_millis(ms),
                    );
                })
                .unwrap();
        }
    }
}
