//! Rolling counters, ported from orangui's `CountUp`: a figure still loading rolls random four-digit numbers, and once its value arrives it lands on it over a second, the digits wild early and still at the end. Figures print in full under ten thousand and as `28.0k`, `1.2m` or `2.5b` above.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

/// The time between frames, from orangui's `COUNT_TICK_MS`.
pub(crate) const TICK: Duration = Duration::from_millis(16);

/// Frames in a landing: orangui's `COUNT_MS` of 1000 in ticks, `ceil(1000 / 16)`.
pub(crate) const FRAMES: u64 = 63;

/// Four figure spaces: what a counter shows before its first frame.
pub(crate) const BLANK: &str = "\u{2007}\u{2007}\u{2007}\u{2007}";

/// Hands each counter its own seed, so neighbors waiting side by side do not roll in step.
static SEEDS: AtomicU64 = AtomicU64::new(1);

/// A fresh seed for a new counter: the next count times 2654435761, as orangui's.
pub(crate) fn next_seed() -> u64 {
    SEEDS
        .fetch_add(1, Ordering::Relaxed)
        .wrapping_mul(2_654_435_761)
}

/// One xorshift round, enough randomness for digits that flicker.
pub(crate) fn noise(seed: u64) -> u64 {
    let mut state = seed.wrapping_mul(0x9e37_79b9_7f4a_7c15) | 1;
    state ^= state << 13;
    state ^= state >> 7;
    state ^= state << 17;
    state
}

/// The number a waiting counter with `seed` shows at `tick`: always four digits.
pub(crate) fn waiting(seed: u64, tick: u64) -> u64 {
    1_000_u64.saturating_add(noise(seed.wrapping_add(tick)) % 9_000)
}

/// What a landing on `value` shows at `frame` of [`FRAMES`]: a random swing that narrows to nothing, never wider than `value`'s digits. The swing seeds on the frame alone, as orangui's does.
#[expect(
    clippy::as_conversions,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "counts sit far below 2^53 so u64 to f64 is exact, and the roll is clamped at zero and rounded before going back to u64"
)]
pub(crate) fn landing(value: u64, frame: u64) -> u64 {
    if frame >= FRAMES || value == 0 {
        return value;
    }
    let fraction = frame as f64 / FRAMES as f64;
    let spread = (1.0 - fraction).powi(2);
    let swing = (noise(frame) % 2001) as f64 / 1000.0 - 1.0;
    let rolled = (value as f64) + (value as f64) * spread * swing;
    (rolled.max(0.0).round() as u64).min(digit_ceiling(value))
}

/// The largest number with as many digits as `value`: 72 gives 99, 3,960 gives 9,999.
pub(crate) fn digit_ceiling(value: u64) -> u64 {
    let mut ceiling = 9_u64;
    while ceiling < value {
        ceiling = ceiling.saturating_mul(10).saturating_add(9);
    }
    ceiling
}

/// `12345` as `12,345`.
pub(crate) fn with_separators(n: u64) -> String {
    let digits = n.to_string();
    let mut out = String::with_capacity(digits.len().saturating_add(digits.len() / 3));
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && digits.len().saturating_sub(i) % 3 == 0 {
            out.push(',');
        }
        out.push(c);
    }
    out
}

/// `n` in full with separators under ten thousand, else truncated to one decimal with `k`, `m` or `b`.
pub(crate) fn compact(n: u64) -> String {
    const STEPS: [(u64, char); 3] = [(1_000_000_000, 'b'), (1_000_000, 'm'), (1_000, 'k')];
    if n < 10_000 {
        return with_separators(n);
    }
    STEPS.iter().find(|(unit, _)| n >= *unit).map_or_else(
        || with_separators(n),
        |&(unit, letter)| {
            let tenths = n / (unit / 10);
            format!("{}.{}{letter}", tenths / 10, tenths % 10)
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compact_keeps_small_numbers_whole_and_shortens_the_rest() {
        assert_eq!(compact(2_065), "2,065");
        assert_eq!(compact(9_999), "9,999");
        assert_eq!(compact(10_000), "10.0k");
        assert_eq!(compact(28_018), "28.0k");
        assert_eq!(compact(244_855), "244.8k");
        assert_eq!(compact(1_234_567), "1.2m");
        assert_eq!(compact(2_500_000_000), "2.5b");
        assert_eq!(compact(u64::MAX), "18446744073.7b");
    }

    #[test]
    fn a_landing_ends_on_its_value_and_keeps_its_width() {
        assert_eq!(landing(3_657, FRAMES), 3_657);
        assert_eq!(landing(0, 1), 0);
        for frame in 1..FRAMES {
            assert!(landing(3_657, frame) <= 9_999, "frame {frame}");
        }
        assert_ne!(
            (1..10).map(|f| landing(3_657, f)).collect::<Vec<_>>(),
            vec![3_657; 9],
            "early frames swing"
        );
        assert_eq!(digit_ceiling(72), 99);
        assert_eq!(digit_ceiling(0), 9);
    }

    #[test]
    fn waiting_counters_roll_four_digits_out_of_step() {
        let a = next_seed();
        let b = next_seed();
        assert_ne!(a, b);
        for tick in 0..100 {
            assert!((1_000..10_000).contains(&waiting(a, tick)));
        }
        assert_ne!(
            (0..5).map(|t| waiting(a, t)).collect::<Vec<_>>(),
            (0..5).map(|t| waiting(b, t)).collect::<Vec<_>>()
        );
        assert_eq!(with_separators(1_234_567), "1,234,567");
        assert_eq!(BLANK.chars().count(), 4);
    }
}
