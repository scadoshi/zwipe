//! Index and time conversions shared by the carousel and the swipe stack.

use chrono::TimeDelta;

/// Converts a position in a rendered list (a page, a dot, a stacked card) to
/// `f64` for layout math.
#[expect(
    clippy::as_conversions,
    clippy::cast_precision_loss,
    reason = "a rendered list's index is far below 2^53, where f64 stops being exact"
)]
pub(crate) fn layout_index(index: usize) -> f64 {
    index as f64
}

/// Whole milliseconds in a gap between two pointer events, as `f64` for
/// velocity math.
#[expect(
    clippy::as_conversions,
    clippy::cast_precision_loss,
    reason = "two clock reads during one gesture are far less than 2^53 ms (285,000 years) apart"
)]
pub(crate) fn gap_millis(gap: TimeDelta) -> f64 {
    gap.num_milliseconds() as f64
}
