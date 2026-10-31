//! The app's entrance effects in cells: `.content-enter` (0.3 s ease-out, fading in from 0.625rem lower), its `-delayed` form (0.15 s later), and the hero's `hero-scope-in` (0.25 s). A terminal has no opacity, so a piece still entering draws one row low and dimmed for the first half, dimmed for the rest, then settles.
//!
//! Each effect is a pure function of the time since its screen mounted, so a screen keeps one instant and redraws on its tick while [`entering`] is true.

use std::time::Duration;

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};

/// `.content-enter`'s length.
pub(crate) const ENTER: Duration = Duration::from_millis(300);

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
    } else if into < length / 2 {
        Phase::Rising
    } else {
        Phase::Fading
    }
}

/// Whether an entrance of `length` after `delay` is still running.
pub(crate) fn entering(elapsed: Duration, delay: Duration, length: Duration) -> bool {
    phase(elapsed, delay, length) != Phase::Settled
}

/// The area a `.content-enter` piece draws into: `None` while hidden, one row lower while rising. The caller clips it to the frame.
pub(crate) fn area(area: Rect, elapsed: Duration, delay: Duration) -> Option<Rect> {
    match phase(elapsed, delay, ENTER) {
        Phase::Hidden => None,
        Phase::Rising => Some(Rect {
            y: area.y.saturating_add(1).min(area.bottom()),
            height: area.height,
            ..area
        }),
        Phase::Fading | Phase::Settled => Some(area),
    }
}

/// Dims what was drawn in `area` while its entrance has yet to settle.
pub(crate) fn fade(
    f: &mut Frame<'_>,
    area: Rect,
    elapsed: Duration,
    delay: Duration,
    length: Duration,
) {
    if matches!(phase(elapsed, delay, length), Phase::Rising | Phase::Fading) {
        let area = area.intersection(f.area());
        f.buffer_mut()
            .set_style(area, Style::new().add_modifier(Modifier::DIM));
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
        assert_eq!(phase(ms(100), DELAY, ENTER), Phase::Hidden);
        assert_eq!(phase(ms(200), DELAY, ENTER), Phase::Rising);
        assert_eq!(phase(ms(350), DELAY, ENTER), Phase::Fading);
        assert_eq!(phase(ms(450), DELAY, ENTER), Phase::Settled);
        assert!(entering(ms(449), DELAY, ENTER));
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
        assert_eq!(
            area(Rect::new(0, 0, 3, 0), ms(10), Duration::ZERO),
            Some(Rect::new(0, 0, 3, 0))
        );
    }
}
