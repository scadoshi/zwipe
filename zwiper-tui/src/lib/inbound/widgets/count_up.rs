//! A rolling counter on screen: orangui's `CountUp` in time, from [`crate::domain::count_up`].
//!
//! A counter starts when its screen's entrance does (after the splash on the first screen). Until its value arrives it rolls random four-digit numbers every 16 ms; once it has one it lands on it over a second; before it starts it shows four figure spaces. A screen showing one reports itself animating while [`CountUp::animating`] holds, so the loop redraws it.

use std::time::Instant;

use crate::domain::count_up::{self, BLANK, FRAMES, TICK};

/// One counter's seed, when it started, and its value once it arrived.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CountUp {
    seed: u64,
    start: Instant,
    value: Option<u64>,
    /// When the value arrived; the landing runs from here or from `start`, whichever is later.
    arrived: Instant,
}

impl CountUp {
    /// A counter that starts rolling at `start`, with `value` already known or still loading.
    pub(crate) fn new(start: Instant, value: Option<u64>) -> Self {
        Self {
            seed: count_up::next_seed(),
            start,
            value,
            arrived: start,
        }
    }

    #[cfg(test)]
    pub(crate) fn value(&self) -> Option<u64> {
        self.value
    }

    /// Takes `value` at `now`: a counter that was waiting, or held another value, lands on it from here.
    pub(crate) fn set(&mut self, value: u64, now: Instant) {
        if self.value != Some(value) {
            self.value = Some(value);
            self.arrived = now;
        }
    }

    /// Frames of 16 ms from `from` to `now`.
    fn frames(from: Instant, now: Instant) -> u64 {
        let elapsed = now.saturating_duration_since(from).as_millis();
        u64::try_from(elapsed.checked_div(TICK.as_millis()).unwrap_or(0)).unwrap_or(u64::MAX)
    }

    /// The number shown at `now`, `None` before the first frame.
    pub(crate) fn number(&self, now: Instant) -> Option<u64> {
        if now < self.start {
            return None;
        }
        let Some(value) = self.value else {
            return Some(count_up::waiting(self.seed, Self::frames(self.start, now)));
        };
        let landing_from = self.arrived.max(self.start);
        let frame = Self::frames(landing_from, now);
        if frame == 0 {
            // The tick before the landing starts still shows the roll, or nothing at the very start.
            return (landing_from > self.start)
                .then(|| count_up::waiting(self.seed, Self::frames(self.start, now)));
        }
        Some(count_up::landing(value, frame))
    }

    /// The figure at `now` as printed: compact, or four figure spaces before the first frame.
    pub(crate) fn shown(&self, now: Instant) -> String {
        self.number(now)
            .map_or_else(|| BLANK.to_owned(), count_up::compact)
    }

    /// Whether the counter is still moving at `now`.
    pub(crate) fn animating(&self, now: Instant) -> bool {
        match self.value {
            None => true,
            Some(_) => Self::frames(self.arrived.max(self.start), now) < FRAMES,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    /// When a landing is over: the last of its frames.
    const DURATION: Duration = Duration::from_millis(16 * 63);

    #[test]
    fn a_counter_is_blank_until_it_starts_then_rolls_until_its_value_lands() {
        let start = Instant::now();
        let mut counter = CountUp::new(start, None);
        assert_eq!(
            counter.shown(
                start
                    .checked_sub(Duration::from_millis(1))
                    .expect("the clock is past a millisecond")
            ),
            BLANK
        );
        let rolling = counter.number(start + TICK * 3).unwrap();
        assert!((1_000..10_000).contains(&rolling));
        assert!(counter.animating(start + Duration::from_secs(9)));

        let arrived = start + Duration::from_millis(500);
        counter.set(28_018, arrived);
        assert!(
            counter.animating(
                (arrived + DURATION)
                    .checked_sub(TICK * 2)
                    .expect("a landing is longer than two ticks")
            )
        );
        assert_eq!(counter.shown(arrived + DURATION), "28.0k");
        assert!(!counter.animating(arrived + DURATION));
    }

    #[test]
    fn a_known_value_lands_from_the_start_and_a_new_one_lands_again() {
        let start = Instant::now();
        let mut counter = CountUp::new(start, Some(2_065));
        assert_eq!(
            counter.shown(start),
            BLANK,
            "nothing before the first frame"
        );
        assert_eq!(counter.shown(start + DURATION), "2,065");
        counter.set(2_065, start + DURATION);
        assert!(
            !counter.animating(start + DURATION),
            "the same value stays put"
        );
        counter.set(3_000, start + DURATION);
        assert!(counter.animating(start + DURATION + TICK));
        assert_eq!(counter.value(), Some(3_000));
    }
}
