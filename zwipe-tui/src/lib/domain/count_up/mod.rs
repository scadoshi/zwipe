//! The rolling figure, ported from orangui's `CountUp`: random digits while a figure waits, then a roll that narrows onto its value over a second.
//!
//! Each frame is a pure function of the time since the figure appeared or arrived, so a screen only keeps two instants and redraws on its tick.

use std::time::Duration;

/// How long a figure takes to land on its value once it arrives.
pub(crate) const ROLL: Duration = Duration::from_millis(1000);

/// The roll's frame length, as in orangui.
const TICK_MS: u128 = 16;

/// Frames in a full roll: `ceil(1000 / 16)`.
const FRAMES: u128 = 63;

/// Shown before the first frame: four figure spaces, so the width holds still.
pub(crate) const BLANK: &str = "\u{2007}\u{2007}\u{2007}\u{2007}";

/// Where one figure is in its roll.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Frame {
    /// Nothing honest to print yet.
    Blank,
    /// A number to print, rolling or landed.
    Number(u64),
}

/// The figure at `since_open` after it appeared and, once it has a value, `since_arrival` after that value arrived. `seed` keeps neighboring figures out of step.
pub(crate) fn frame(
    value: Option<u64>,
    since_open: Duration,
    since_arrival: Duration,
    seed: u64,
) -> Frame {
    let Some(value) = value else {
        let tick = since_open.as_millis() / TICK_MS;
        if tick == 0 {
            return Frame::Blank;
        }
        let tick = u64::try_from(tick).unwrap_or(u64::MAX);
        return Frame::Number(waiting(noise(seed.wrapping_add(tick))));
    };
    let frame = (since_arrival.as_millis() / TICK_MS).min(FRAMES);
    if frame >= FRAMES {
        return Frame::Number(value);
    }
    let fraction = to_f64(frame) / to_f64(FRAMES);
    Frame::Number(rolling(value, fraction, u64::try_from(frame).unwrap_or(0)))
}

#[expect(
    clippy::as_conversions,
    clippy::cast_precision_loss,
    reason = "frame counts are at most 63, exact in f64"
)]
fn to_f64(n: u128) -> f64 {
    n as f64
}

/// Digits for a figure that has not arrived: four of them, so the width holds still while it waits.
fn waiting(seed: u64) -> u64 {
    1_000_u64.saturating_add(seed % 9_000)
}

/// `value` with a random offset that shrinks to nothing as `fraction` reaches 1, held inside the digit count `value` prints at.
#[expect(
    clippy::as_conversions,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "counts sit far below 2^53 so u64 to f64 is exact, and the roll is clamped at zero and rounded before going back to u64"
)]
fn rolling(value: u64, fraction: f64, seed: u64) -> u64 {
    let fraction = fraction.clamp(0.0, 1.0);
    if fraction >= 1.0 || value == 0 {
        return value;
    }
    let spread = (1.0 - fraction).powi(2);
    let swing = (noise(seed) % 2001) as f64 / 1000.0 - 1.0;
    let rolled = (value as f64) + (value as f64) * spread * swing;
    (rolled.max(0.0).round() as u64).min(digit_ceiling(value))
}

/// The largest number of the same digit count: 72 gives 99, 3,960 gives 9,999.
fn digit_ceiling(value: u64) -> u64 {
    let mut ceiling = 9;
    while ceiling < value {
        ceiling = ceiling.saturating_mul(10).saturating_add(9);
    }
    ceiling
}

/// One xorshift round, enough randomness for digits that flicker.
fn noise(seed: u64) -> u64 {
    let mut state = seed.wrapping_mul(0x9e37_79b9_7f4a_7c15) | 1;
    state ^= state << 13;
    state ^= state >> 7;
    state ^= state << 17;
    state
}

/// `12345` as `12,345`.
pub(crate) fn with_separators(n: u64) -> String {
    let digits = n.to_string();
    let mut out = String::with_capacity(digits.len().saturating_add(digits.len() / 3));
    for (i, ch) in digits.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            out.push(',');
        }
        out.push(ch);
    }
    out.chars().rev().collect()
}

/// `n` to one decimal with a magnitude letter from ten thousand up (`28.0k`, `1.2m`, `3.4b`), truncated so a figure never reads past what it is. Under ten thousand, [`with_separators`].
pub(crate) fn short(n: u64) -> String {
    const STEPS: [(u64, char); 3] = [(1_000_000_000, 'b'), (1_000_000, 'm'), (1_000, 'k')];
    if n < 10_000 {
        return with_separators(n);
    }
    for (unit, letter) in STEPS {
        if n >= unit {
            let tenths = n / (unit / 10);
            return format!("{}.{}{letter}", tenths / 10, tenths % 10);
        }
    }
    with_separators(n)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compact_figures_truncate_from_ten_thousand() {
        assert_eq!(short(0), "0");
        assert_eq!(short(9_999), "9,999");
        assert_eq!(short(10_000), "10.0k");
        assert_eq!(short(28_049), "28.0k");
        assert_eq!(short(1_299_999), "1.2m");
        assert_eq!(short(3_456_000_000), "3.4b");
        assert_eq!(with_separators(1_234_567), "1,234,567");
    }

    #[test]
    fn a_waiting_figure_is_blank_then_rolls_four_digits() {
        assert_eq!(frame(None, Duration::ZERO, Duration::ZERO, 1), Frame::Blank);
        for ms in [16, 160, 4_000] {
            let Frame::Number(n) = frame(None, Duration::from_millis(ms), Duration::ZERO, 7) else {
                panic!("rolls once a frame has passed");
            };
            assert!((1_000..=9_999).contains(&n));
        }
    }

    #[test]
    fn an_arrived_figure_stays_in_its_digits_and_lands_after_a_second() {
        for ms in (0..1_000).step_by(16) {
            let Frame::Number(n) = frame(Some(72), Duration::ZERO, Duration::from_millis(ms), 3)
            else {
                panic!("a value always prints");
            };
            assert!(n <= 99, "{n} at {ms}ms");
        }
        assert_eq!(frame(Some(72), Duration::ZERO, ROLL, 3), Frame::Number(72));
        assert_eq!(
            frame(Some(0), Duration::ZERO, Duration::ZERO, 3),
            Frame::Number(0)
        );
        assert_eq!(digit_ceiling(3_960), 9_999);
    }
}
