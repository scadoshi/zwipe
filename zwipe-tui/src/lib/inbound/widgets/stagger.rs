//! Staggered entrances inside a screen, on top of [`super::entrance`]: `.content-enter-delayed` (the same 0.3 s, 0.15 s later), the rise from 0.625rem lower, and the hero's `hero-scope-in` (0.25 s). A terminal has no opacity, so a piece still entering draws one row low and dimmed for its first half, dimmed for the rest, then settles.
//!
//! Each effect is a pure function of the time since its screen mounted, so a screen keeps one instant and redraws on its tick while [`entering`] is true.

use std::time::Duration;

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};

use super::entrance;

/// `.content-enter-delayed`'s extra wait.
pub(crate) const DELAY: Duration = Duration::from_millis(150);

/// `hero-scope-in`'s length, for the scope label and the Live chip.
pub(crate) const SCOPE_IN: Duration = Duration::from_millis(250);

/// Where a piece is in its entrance.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Phase {
    /// Still waiting on its delay: not drawn.
    Hidden,
    /// The first half: one row low and dim.
    Rising,
    /// The second half: in place, still dim.
    Fading,
    Settled,
}

/// The phase of an entrance of `length` that waits `delay`, `elapsed` after mount.
pub(crate) fn phase(elapsed: Duration, delay: Duration, length: Duration) -> Phase {
    let Some(into) = elapsed.checked_sub(delay) else {
        return Phase::Hidden;
    };
    if into >= length {
        Phase::Settled
    } else if into < length.checked_div(2).unwrap_or(length) {
        Phase::Rising
    } else {
        Phase::Fading
    }
}

/// Whether an entrance of `length` after `delay` is still running.
pub(crate) fn entering(elapsed: Duration, delay: Duration, length: Duration) -> bool {
    match elapsed.checked_sub(delay) {
        None => true,
        Some(into) if length == entrance::DURATION => entrance::animating(into),
        Some(into) => into < length,
    }
}

/// The area a `.content-enter` piece draws into: `None` while hidden, one row lower while rising. The caller clips it to the frame.
pub(crate) fn area(area: Rect, elapsed: Duration, delay: Duration) -> Option<Rect> {
    match phase(elapsed, delay, entrance::DURATION) {
        Phase::Hidden => None,
        Phase::Rising => Some(Rect {
            y: area.y.saturating_add(1).min(area.bottom()),
            ..area
        }),
        Phase::Fading | Phase::Settled => Some(area),
    }
}

/// Dims what was drawn in `area` while its entrance runs: [`entrance::draw`]'s faint first half for a content entrance, the whole length for any other.
pub(crate) fn fade(
    f: &mut Frame<'_>,
    area: Rect,
    elapsed: Duration,
    delay: Duration,
    length: Duration,
) {
    let area = area.intersection(f.area());
    match phase(elapsed, delay, length) {
        Phase::Rising | Phase::Fading if length == entrance::DURATION => {
            entrance::draw(f, area, elapsed.saturating_sub(delay));
        }
        Phase::Rising | Phase::Fading => {
            f.buffer_mut()
                .set_style(area, Style::new().add_modifier(Modifier::DIM));
        }
        Phase::Hidden | Phase::Settled => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ms(n: u64) -> Duration {
        Duration::from_millis(n)
    }

    #[test]
    fn a_delayed_piece_waits_rises_fades_and_settles() {
        let length = entrance::DURATION;
        assert_eq!(phase(ms(100), DELAY, length), Phase::Hidden);
        assert_eq!(phase(ms(200), DELAY, length), Phase::Rising);
        assert_eq!(phase(ms(350), DELAY, length), Phase::Fading);
        assert_eq!(phase(ms(450), DELAY, length), Phase::Settled);
        assert!(entering(ms(449), DELAY, length));
        assert!(!entering(ms(250), Duration::ZERO, SCOPE_IN));
    }

    #[test]
    fn a_rising_piece_draws_a_row_low() {
        let rect = Rect::new(0, 5, 10, 4);
        assert_eq!(area(rect, ms(0), DELAY), None);
        assert_eq!(
            area(rect, ms(10), Duration::ZERO),
            Some(Rect::new(0, 6, 10, 4))
        );
        assert_eq!(area(rect, ms(300), Duration::ZERO), Some(rect));
    }
}
