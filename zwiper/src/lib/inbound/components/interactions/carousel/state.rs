//! Carousel state management.
//!
//! Tracks the current page index, drag offset, and velocity for a horizontal
//! snap-to-page carousel. Unlike [`SwipeState`](super::super::swipe::state::SwipeState)
//! which detects fling gestures, this state manages persistent page position
//! with smooth snapping and edge resistance.

use chrono::{NaiveDateTime, Utc};

/// Tracks the state of a horizontal snap-to-page carousel.
#[derive(Debug, Clone)]
pub struct CarouselState {
    /// Which page is currently displayed.
    pub current_index: usize,
    /// Total number of pages.
    pub page_count: usize,
    /// Width of one page in pixels (measured from viewport).
    pub page_width_px: f64,
    /// Current drag displacement in pixels during an active gesture.
    pub drag_offset_px: f64,
    /// Whether a drag gesture is currently in progress.
    pub is_dragging: bool,
    /// Transition duration in ms: 0 during drag, 300 on snap.
    pub snap_transition_ms: u64,
    /// X coordinate at drag start.
    start_x: Option<f64>,
    /// Previous X coordinate for velocity calculation.
    prev_x: Option<f64>,
    /// Timestamp of the previous move event.
    prev_time: Option<NaiveDateTime>,
    /// Horizontal velocity in px/ms at the moment of release.
    velocity_px_ms: f64,
}

/// Minimum drag ratio to trigger a page advance (30% of page width).
const SNAP_DISTANCE_RATIO: f64 = 0.3;

/// Minimum drag ratio when velocity is also present (5% of page width).
const SNAP_DISTANCE_WITH_VELOCITY_RATIO: f64 = 0.05;

/// Minimum velocity in px/ms to assist a page advance.
const VELOCITY_THRESHOLD: f64 = 0.5;

/// Dampening factor for edge resistance rubber-band effect.
const EDGE_DAMPENING: f64 = 3.0;

/// Duration in ms for the snap-back/snap-forward animation.
const SNAP_DURATION_MS: u64 = 300;

impl CarouselState {
    /// Creates a new carousel state with the given configuration.
    pub fn new(page_count: usize, start_index: usize, page_width_px: f64) -> Self {
        Self {
            current_index: start_index.min(page_count.saturating_sub(1)),
            page_count,
            page_width_px,
            drag_offset_px: 0.0,
            is_dragging: false,
            snap_transition_ms: 0,
            start_x: None,
            prev_x: None,
            prev_time: None,
            velocity_px_ms: 0.0,
        }
    }

    /// Creates an empty state suitable for signal initialization before data loads.
    pub fn empty() -> Self {
        Self::new(0, 0, 0.0)
    }

    /// The CSS `translateX` for the carousel strip: the resting page position as
    /// a **percentage** plus the live pixel drag offset.
    ///
    /// The flex strip's own box is exactly one viewport wide (each page is
    /// `flex: 0 0 100%` and the extra pages *overflow* the strip, clipped by the
    /// viewport's `overflow: hidden`), so `100%` of the strip equals one page —
    /// translating by `index * 100%` lands on page `index`. This is drift-proof:
    /// unlike a pixel-per-page translate off a measured `page_width_px` (whose
    /// error accumulates as `index * error` and veers sideways more with each
    /// page — the printing-carousel drift), the percentage needs no measurement.
    /// `page_width_px` is still used for snap *thresholds*, where a small error is
    /// harmless.
    pub fn translate_x_css(&self) -> String {
        let percent = if self.current_index > 0 {
            -(self.current_index as f64) * 100.0
        } else {
            0.0
        };
        if self.drag_offset_px >= 0.0 {
            format!("calc({percent}% + {}px)", self.drag_offset_px)
        } else {
            format!("calc({percent}% - {}px)", -self.drag_offset_px)
        }
    }

    /// Records the start of a drag gesture.
    pub fn on_drag_start(&mut self, client_x: f64) {
        self.start_x = Some(client_x);
        self.prev_x = Some(client_x);
        self.prev_time = Some(Utc::now().naive_utc());
        self.velocity_px_ms = 0.0;
        self.is_dragging = true;
        self.snap_transition_ms = 0;
    }

    /// Updates the drag offset during an active gesture.
    pub fn on_drag_move(&mut self, client_x: f64) {
        let Some(start) = self.start_x else { return };

        let raw_delta = client_x - start;

        // Edge resistance: rubber-band when dragging past first or last page
        let at_start = self.current_index == 0 && raw_delta > 0.0;
        let at_end =
            self.page_count > 0 && self.current_index >= self.page_count - 1 && raw_delta < 0.0;

        if self.page_width_px > 0.0 && (at_start || at_end) {
            self.drag_offset_px =
                raw_delta / (1.0 + (raw_delta.abs() / self.page_width_px) * EDGE_DAMPENING);
        } else {
            self.drag_offset_px = raw_delta;
        }

        // Update velocity tracking
        let now = Utc::now().naive_utc();
        if let (Some(px), Some(pt)) = (self.prev_x, self.prev_time) {
            let dt_ms = (now - pt).num_milliseconds() as f64;
            if dt_ms > 0.0 {
                self.velocity_px_ms = (client_x - px) / dt_ms;
            }
        }
        self.prev_x = Some(client_x);
        self.prev_time = Some(now);
    }

    /// Completes a drag gesture: decides the target page and initiates snap.
    pub fn on_drag_end(&mut self, _client_x: f64) {
        if self.page_count == 0 || self.page_width_px == 0.0 {
            self.reset_drag();
            return;
        }

        let offset_ratio = self.drag_offset_px / self.page_width_px;
        let velocity_abs = self.velocity_px_ms.abs();

        let should_advance = offset_ratio.abs() > SNAP_DISTANCE_RATIO
            || (offset_ratio.abs() > SNAP_DISTANCE_WITH_VELOCITY_RATIO
                && velocity_abs > VELOCITY_THRESHOLD);

        if should_advance {
            if self.drag_offset_px < 0.0 && self.current_index < self.page_count - 1 {
                // Swiped left → next page
                self.current_index += 1;
            } else if self.drag_offset_px > 0.0 && self.current_index > 0 {
                // Swiped right → previous page
                self.current_index -= 1;
            }
        }

        self.reset_drag();
        self.snap_transition_ms = SNAP_DURATION_MS;
    }

    /// Clears drag-related fields without changing `current_index`.
    fn reset_drag(&mut self) {
        self.drag_offset_px = 0.0;
        self.is_dragging = false;
        self.start_x = None;
        self.prev_x = None;
        self.prev_time = None;
        self.velocity_px_ms = 0.0;
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    // ── construction ────────────────────────────────────────────────────────

    #[test]
    fn new_clamps_start_index_to_last_page() {
        let state = CarouselState::new(3, 10, 375.0);
        assert_eq!(state.current_index, 2);
    }

    #[test]
    fn empty_has_zero_pages() {
        let state = CarouselState::empty();
        assert_eq!(state.page_count, 0);
        assert_eq!(state.current_index, 0);
    }

    // ── translate_x_css (percentage-of-strip, drift-proof) ─────────────────

    #[test]
    fn translate_at_rest_page_zero() {
        let state = CarouselState::new(5, 0, 375.0);
        assert_eq!(state.translate_x_css(), "calc(0% + 0px)");
    }

    #[test]
    fn translate_at_rest_page_two() {
        // page 2 => -2 * 100% = -200% (the strip box is one viewport wide, so
        // 100% == one page; pages overflow and are clipped by the viewport)
        let state = CarouselState::new(5, 2, 375.0);
        assert_eq!(state.translate_x_css(), "calc(-200% + 0px)");
    }

    #[test]
    fn translate_with_drag_offset() {
        let mut state = CarouselState::new(5, 1, 375.0);
        state.drag_offset_px = -100.0;
        // page 1 resting (-100%) minus the 100px live drag
        assert_eq!(state.translate_x_css(), "calc(-100% - 100px)");
    }

    // ── snap logic ──────────────────────────────────────────────────────────

    #[test]
    fn snap_forward_when_dragged_past_threshold() {
        let mut state = CarouselState::new(5, 1, 375.0);
        state.on_drag_start(200.0);
        // Drag left by 40% of page width (past 30% threshold)
        state.drag_offset_px = -150.0; // 150/375 = 0.4
        state.on_drag_end(50.0);
        assert_eq!(state.current_index, 2);
        assert_eq!(state.drag_offset_px, 0.0);
        assert_eq!(state.snap_transition_ms, SNAP_DURATION_MS);
    }

    #[test]
    fn snap_back_when_below_threshold() {
        let mut state = CarouselState::new(5, 1, 375.0);
        state.on_drag_start(200.0);
        // Drag left by only 10% (below 30% threshold, no velocity)
        state.drag_offset_px = -37.5;
        state.velocity_px_ms = 0.0;
        state.on_drag_end(162.5);
        assert_eq!(state.current_index, 1); // unchanged
    }

    #[test]
    fn snap_forward_via_velocity() {
        let mut state = CarouselState::new(5, 1, 375.0);
        state.on_drag_start(200.0);
        // Small drag (6%) but high velocity
        state.drag_offset_px = -22.5; // 22.5/375 = 0.06
        state.velocity_px_ms = -0.8; // fast leftward
        state.on_drag_end(177.5);
        assert_eq!(state.current_index, 2);
    }

    #[test]
    fn snap_backward_when_dragged_right() {
        let mut state = CarouselState::new(5, 2, 375.0);
        state.on_drag_start(100.0);
        state.drag_offset_px = 150.0; // 40% rightward
        state.on_drag_end(250.0);
        assert_eq!(state.current_index, 1);
    }

    #[test]
    fn clamp_at_first_page() {
        let mut state = CarouselState::new(5, 0, 375.0);
        state.on_drag_start(100.0);
        state.drag_offset_px = 200.0; // trying to go before page 0
        state.on_drag_end(300.0);
        assert_eq!(state.current_index, 0); // stays at 0
    }

    #[test]
    fn clamp_at_last_page() {
        let mut state = CarouselState::new(5, 4, 375.0);
        state.on_drag_start(200.0);
        state.drag_offset_px = -200.0; // trying to go past last page
        state.on_drag_end(0.0);
        assert_eq!(state.current_index, 4); // stays at 4
    }

    // ── edge resistance ─────────────────────────────────────────────────────

    #[test]
    fn edge_resistance_at_first_page() {
        let mut state = CarouselState::new(5, 0, 375.0);
        state.on_drag_start(100.0);
        // Drag right (past start boundary)
        state.on_drag_move(300.0); // raw_delta = 200
        // Should be dampened: 200 / (1 + (200/375)*3) = 200 / 2.6 ≈ 76.9
        assert!(state.drag_offset_px > 0.0);
        assert!(state.drag_offset_px < 100.0); // significantly dampened from 200
    }

    #[test]
    fn no_resistance_in_middle() {
        let mut state = CarouselState::new(5, 2, 375.0);
        state.on_drag_start(200.0);
        state.on_drag_move(100.0); // raw_delta = -100 (left, but not at boundary)
        assert_eq!(state.drag_offset_px, -100.0); // no dampening
    }

    #[test]
    fn edge_resistance_at_last_page() {
        let mut state = CarouselState::new(5, 4, 375.0);
        state.on_drag_start(200.0);
        // Drag left (past end boundary)
        state.on_drag_move(0.0); // raw_delta = -200
        assert!(state.drag_offset_px < 0.0);
        assert!(state.drag_offset_px > -100.0); // dampened from -200
    }

    // ── drag lifecycle ──────────────────────────────────────────────────────

    #[test]
    fn drag_start_clears_transition() {
        let mut state = CarouselState::new(5, 1, 375.0);
        state.snap_transition_ms = 300;
        state.on_drag_start(100.0);
        assert_eq!(state.snap_transition_ms, 0);
        assert!(state.is_dragging);
    }

    #[test]
    fn drag_end_sets_transition() {
        let mut state = CarouselState::new(5, 1, 375.0);
        state.on_drag_start(100.0);
        state.on_drag_end(100.0);
        assert_eq!(state.snap_transition_ms, SNAP_DURATION_MS);
        assert!(!state.is_dragging);
    }

    #[test]
    fn drag_end_with_empty_state_does_not_panic() {
        let mut state = CarouselState::empty();
        state.on_drag_start(100.0);
        state.on_drag_end(50.0); // should not panic
        assert_eq!(state.current_index, 0);
    }
}
