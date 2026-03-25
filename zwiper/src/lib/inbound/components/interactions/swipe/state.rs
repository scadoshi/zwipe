//! Swipe state management.
//!
//! Tracks the current state of a swipe gesture including position, velocity,
//! and the final swipe direction when the gesture completes.

use crate::inbound::components::interactions::swipe::{
    axis::Axis, config::SwipeConfig, direction::Direction as Dir, time_point::TimePoint,
};
use dioxus::html::geometry::euclid::{Point2D, UnknownUnit};

type DeltaPoint = Point2D<f64, UnknownUnit>;

/// Tracks the state of an ongoing swipe gesture.
#[derive(Debug, Clone)]
pub struct SwipeState {
    /// Starting position and timestamp of the swipe.
    pub start_point: Option<TimePoint>,
    /// Previous position for velocity calculation.
    pub previous_point: Option<TimePoint>,
    /// Current touch/mouse position.
    pub current_point: Option<TimePoint>,
    /// The direction the last completed swipe resolved to.
    pub latest_swipe: Option<Dir>,
    /// Which axis the user is currently swiping along (locked after initial movement).
    pub traversing_axis: Option<Axis>,
    /// Whether a swipe gesture is currently in progress.
    pub is_swiping: bool,
    /// Duration for the return-to-origin animation in seconds.
    pub return_animation_seconds: f64,
}

impl Default for SwipeState {
    fn default() -> Self {
        Self {
            start_point: None,
            current_point: None,
            previous_point: None,
            latest_swipe: None,
            traversing_axis: None,
            is_swiping: false,
            return_animation_seconds: 0.0,
        }
    }
}

impl SwipeState {
    /// Creates a new SwipeState with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Resets the swipe state, clearing all tracked points and axis.
    pub fn reset(&mut self) {
        self.start_point = None;
        self.current_point = None;
        self.previous_point = None;
        self.traversing_axis = None;
        self.is_swiping = false;
    }

    /// Calculates the Euclidean distance from the start point to the current point.
    pub fn distance_from_start_point(&self) -> Option<f64> {
        let (Some(start), Some(curr)) = (self.start_point.as_ref(), self.current_point.as_ref())
        else {
            return None;
        };

        let dx = curr.point.x - start.point.x;
        let dy = curr.point.y - start.point.y;

        Some((dx.powi(2) + dy.powi(2)).sqrt())
    }

    /// Returns the (x, y) delta from the start point to the current point.
    pub fn delta_from_start_point(&self) -> Option<DeltaPoint> {
        let (Some(start), Some(curr)) = (self.start_point.as_ref(), self.current_point.as_ref())
        else {
            return None;
        };
        Some(DeltaPoint::new(
            curr.point.x - start.point.x,
            curr.point.y - start.point.y,
        ))
    }

    /// Calculates the distance from the previous point to the current point.
    pub fn distance_from_previous_point(&self) -> Option<f64> {
        let (Some(prev), Some(curr)) = (self.previous_point.as_ref(), self.current_point.as_ref())
        else {
            return None;
        };

        let dx = curr.point.x - prev.point.x;
        let dy = curr.point.y - prev.point.y;

        Some((dx.powi(2) + dy.powi(2)).sqrt())
    }

    /// Returns the (x, y) delta from the previous point to the current point.
    pub fn delta_from_previous_point(&self) -> Option<DeltaPoint> {
        let (Some(prev), Some(curr)) = (self.previous_point.as_ref(), self.current_point.as_ref())
        else {
            return None;
        };
        Some(DeltaPoint::new(
            curr.point.x - prev.point.x,
            curr.point.y - prev.point.y,
        ))
    }

    /// Returns the time elapsed in milliseconds between the previous and current points.
    pub fn milliseconds_from_previous_point(&self) -> Option<f64> {
        let (Some(prev), Some(curr)) = (self.previous_point.as_ref(), self.current_point.as_ref())
        else {
            return None;
        };
        Some((curr.time - prev.time).num_milliseconds() as f64)
    }

    /// Calculates the current swipe speed in pixels per millisecond.
    pub fn speed(&self) -> Option<f64> {
        let (Some(distance), Some(time)) = (
            self.distance_from_previous_point(),
            self.milliseconds_from_previous_point(),
        ) else {
            return None;
        };
        Some(distance / time)
    }

    /// Calculates the return animation duration based on swipe distance.
    ///
    /// Longer swipes result in longer animation times for a smooth return.
    pub fn calculate_return_animation_seconds(&mut self) {
        let mut s = 0.0;

        if let Some(d) = self.distance_from_start_point() {
            if d > 0.0 {
                s = 0.25;
            }
            if d > 25.0 {
                s = 0.5;
            }
            if d > 50.0 {
                s = 0.625;
            }
            if d > 100.0 {
                s = 0.75;
            }
        }

        self.return_animation_seconds = s;
    }

    /// Evaluates the current swipe and sets `latest_swipe` if thresholds are met.
    pub fn set_latest_swipe(&mut self, config: &SwipeConfig) {
        const DISTANCE_THRESHOLD_FOR_SPEED_TO_BE_VALID: f64 = 10.0;

        if self.traversing_axis.is_none() {
            self.set_traversing_axis();
        }

        let (Some(distance_from_start), Some(speed)) =
            (self.distance_from_start_point(), self.speed())
        else {
            return;
        };

        if distance_from_start > config.distance_threshold
            || (distance_from_start > DISTANCE_THRESHOLD_FOR_SPEED_TO_BE_VALID
                && speed > config.speed_threshold)
        {
            match self.traversing_axis {
                Some(Axis::X) => {
                    let Some(DeltaPoint { x, .. }) = self.delta_from_start_point() else {
                        return;
                    };

                    if x < 0.0 {
                        let direction = Dir::Left;
                        if config.allowed_directions.contains(&direction) {
                            self.latest_swipe = Some(direction);
                        }
                    }
                    if x > 0.0 {
                        let direction = Dir::Right;
                        if config.allowed_directions.contains(&direction) {
                            self.latest_swipe = Some(direction);
                        }
                    }
                }

                Some(Axis::Y) => {
                    let Some(DeltaPoint { y, .. }) = self.delta_from_start_point() else {
                        return;
                    };

                    if y < 0.0 {
                        let direction = Dir::Up;
                        if config.allowed_directions.contains(&direction) {
                            self.latest_swipe = Some(direction);
                        }
                    }
                    if y > 0.0 {
                        let direction = Dir::Down;
                        if config.allowed_directions.contains(&direction) {
                            self.latest_swipe = Some(direction);
                        }
                    }
                }

                None => (),
            }
        }

        tracing::trace!("swipe dir={:?}", self.latest_swipe);
    }

    /// Determines and locks the swipe axis (X or Y) based on initial movement.
    pub fn set_traversing_axis(&mut self) {
        let (Some(start), Some(curr)) = (self.start_point.as_ref(), self.current_point.as_ref())
        else {
            return;
        };

        let dx = (curr.point.x - start.point.x).abs();
        let dy = (curr.point.y - start.point.y).abs();

        if dx > dy {
            self.traversing_axis = Some(Axis::X);
        }

        if dy > dx {
            self.traversing_axis = Some(Axis::Y);
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::inbound::components::interactions::swipe::{
        config::SwipeConfig, direction::Direction, time_point::TimePoint,
    };
    use chrono::NaiveDate;
    use dioxus::html::geometry::euclid::Point2D;

    /// Builds a TimePoint at (x, y) offset by `ms` milliseconds from a fixed base time.
    fn tp(x: f64, y: f64, ms: i64) -> TimePoint {
        let base = NaiveDate::from_ymd_opt(2024, 1, 1)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap();
        TimePoint::new(
            Point2D::new(x, y),
            base + chrono::Duration::milliseconds(ms),
        )
    }

    // ── distance_from_start_point ─────────────────────────────────────────────

    #[test]
    fn test_distance_from_start_none_when_no_points() {
        let state = SwipeState::default();
        assert!(state.distance_from_start_point().is_none());
    }

    #[test]
    fn test_distance_from_start_pure_horizontal() {
        let state = SwipeState {
            start_point: Some(tp(0.0, 0.0, 0)),
            current_point: Some(tp(30.0, 0.0, 100)),
            ..SwipeState::default()
        };
        assert_eq!(state.distance_from_start_point().unwrap(), 30.0);
    }

    #[test]
    fn test_distance_from_start_pure_vertical() {
        let state = SwipeState {
            start_point: Some(tp(0.0, 0.0, 0)),
            current_point: Some(tp(0.0, 40.0, 100)),
            ..SwipeState::default()
        };
        assert_eq!(state.distance_from_start_point().unwrap(), 40.0);
    }

    #[test]
    fn test_distance_from_start_diagonal() {
        // 3-4-5 right triangle → hypotenuse = 5.0
        let state = SwipeState {
            start_point: Some(tp(0.0, 0.0, 0)),
            current_point: Some(tp(3.0, 4.0, 100)),
            ..SwipeState::default()
        };
        assert_eq!(state.distance_from_start_point().unwrap(), 5.0);
    }

    // ── delta_from_start_point ────────────────────────────────────────────────

    #[test]
    fn test_delta_from_start_positive() {
        let state = SwipeState {
            start_point: Some(tp(10.0, 20.0, 0)),
            current_point: Some(tp(30.0, 50.0, 100)),
            ..SwipeState::default()
        };
        let delta = state.delta_from_start_point().unwrap();
        assert_eq!(delta.x, 20.0);
        assert_eq!(delta.y, 30.0);
    }

    #[test]
    fn test_delta_from_start_negative() {
        let state = SwipeState {
            start_point: Some(tp(30.0, 50.0, 0)),
            current_point: Some(tp(10.0, 20.0, 100)),
            ..SwipeState::default()
        };
        let delta = state.delta_from_start_point().unwrap();
        assert_eq!(delta.x, -20.0);
        assert_eq!(delta.y, -30.0);
    }

    // ── distance_from_previous_point ──────────────────────────────────────────

    #[test]
    fn test_distance_from_previous_none_when_missing() {
        let state = SwipeState::default();
        assert!(state.distance_from_previous_point().is_none());
    }

    #[test]
    fn test_distance_from_previous_3_4_5() {
        let state = SwipeState {
            previous_point: Some(tp(0.0, 0.0, 0)),
            current_point: Some(tp(3.0, 4.0, 100)),
            ..SwipeState::default()
        };
        assert_eq!(state.distance_from_previous_point().unwrap(), 5.0);
    }

    // ── milliseconds_from_previous_point ──────────────────────────────────────

    #[test]
    fn test_millis_none_when_missing() {
        let state = SwipeState::default();
        assert!(state.milliseconds_from_previous_point().is_none());
    }

    #[test]
    fn test_millis_elapsed() {
        let state = SwipeState {
            previous_point: Some(tp(0.0, 0.0, 0)),
            current_point: Some(tp(0.0, 0.0, 100)),
            ..SwipeState::default()
        };
        assert_eq!(state.milliseconds_from_previous_point().unwrap(), 100.0);
    }

    // ── speed ─────────────────────────────────────────────────────────────────

    #[test]
    fn test_speed_none_when_missing() {
        let state = SwipeState::default();
        assert!(state.speed().is_none());
    }

    #[test]
    fn test_speed_calculation() {
        // 50px in 10ms = 5.0 px/ms
        let state = SwipeState {
            previous_point: Some(tp(0.0, 0.0, 0)),
            current_point: Some(tp(50.0, 0.0, 10)),
            ..SwipeState::default()
        };
        assert_eq!(state.speed().unwrap(), 5.0);
    }

    #[test]
    fn test_speed_zero_time_gives_infinity() {
        // 50px in 0ms → 50/0 = f64::INFINITY
        let state = SwipeState {
            previous_point: Some(tp(0.0, 0.0, 0)),
            current_point: Some(tp(50.0, 0.0, 0)),
            ..SwipeState::default()
        };
        assert!(state.speed().unwrap().is_infinite());
    }

    // ── calculate_return_animation_seconds ────────────────────────────────────

    #[test]
    fn test_animation_zero_when_no_points() {
        let mut state = SwipeState::default();
        state.calculate_return_animation_seconds();
        assert_eq!(state.return_animation_seconds, 0.0);
    }

    #[test]
    fn test_animation_0_25_at_1px() {
        let mut state = SwipeState {
            start_point: Some(tp(0.0, 0.0, 0)),
            current_point: Some(tp(1.0, 0.0, 100)),
            ..SwipeState::default()
        };
        state.calculate_return_animation_seconds();
        assert_eq!(state.return_animation_seconds, 0.25);
    }

    #[test]
    fn test_animation_boundary_exactly_25() {
        // distance = 25.0, which is NOT > 25, so stays at 0.25
        let mut state = SwipeState {
            start_point: Some(tp(0.0, 0.0, 0)),
            current_point: Some(tp(25.0, 0.0, 100)),
            ..SwipeState::default()
        };
        state.calculate_return_animation_seconds();
        assert_eq!(state.return_animation_seconds, 0.25);
    }

    #[test]
    fn test_animation_0_5_at_26px() {
        let mut state = SwipeState {
            start_point: Some(tp(0.0, 0.0, 0)),
            current_point: Some(tp(26.0, 0.0, 100)),
            ..SwipeState::default()
        };
        state.calculate_return_animation_seconds();
        assert_eq!(state.return_animation_seconds, 0.5);
    }

    #[test]
    fn test_animation_0_625_at_51px() {
        let mut state = SwipeState {
            start_point: Some(tp(0.0, 0.0, 0)),
            current_point: Some(tp(51.0, 0.0, 100)),
            ..SwipeState::default()
        };
        state.calculate_return_animation_seconds();
        assert_eq!(state.return_animation_seconds, 0.625);
    }

    #[test]
    fn test_animation_boundary_exactly_100() {
        // distance = 100.0, which is NOT > 100, so stays at 0.625
        let mut state = SwipeState {
            start_point: Some(tp(0.0, 0.0, 0)),
            current_point: Some(tp(100.0, 0.0, 100)),
            ..SwipeState::default()
        };
        state.calculate_return_animation_seconds();
        assert_eq!(state.return_animation_seconds, 0.625);
    }

    #[test]
    fn test_animation_0_75_at_101px() {
        let mut state = SwipeState {
            start_point: Some(tp(0.0, 0.0, 0)),
            current_point: Some(tp(101.0, 0.0, 100)),
            ..SwipeState::default()
        };
        state.calculate_return_animation_seconds();
        assert_eq!(state.return_animation_seconds, 0.75);
    }

    // ── set_traversing_axis ───────────────────────────────────────────────────

    #[test]
    fn test_axis_x_when_horizontal_dominates() {
        let mut state = SwipeState {
            start_point: Some(tp(0.0, 0.0, 0)),
            current_point: Some(tp(10.0, 3.0, 100)),
            ..SwipeState::default()
        };
        state.set_traversing_axis();
        assert_eq!(state.traversing_axis, Some(Axis::X));
    }

    #[test]
    fn test_axis_y_when_vertical_dominates() {
        let mut state = SwipeState {
            start_point: Some(tp(0.0, 0.0, 0)),
            current_point: Some(tp(3.0, 10.0, 100)),
            ..SwipeState::default()
        };
        state.set_traversing_axis();
        assert_eq!(state.traversing_axis, Some(Axis::Y));
    }

    #[test]
    fn test_axis_none_when_equal_deltas() {
        // dx == dy → neither branch fires, axis stays None
        let mut state = SwipeState {
            start_point: Some(tp(0.0, 0.0, 0)),
            current_point: Some(tp(5.0, 5.0, 100)),
            ..SwipeState::default()
        };
        state.set_traversing_axis();
        assert!(state.traversing_axis.is_none());
    }

    #[test]
    fn test_axis_noop_when_no_points() {
        let mut state = SwipeState::default();
        state.set_traversing_axis();
        assert!(state.traversing_axis.is_none());
    }

    // ── set_latest_swipe ──────────────────────────────────────────────────────

    fn default_config() -> SwipeConfig {
        SwipeConfig::default() // distance=100.0, speed=5.0, all directions
    }

    /// Builds a state that has moved `dx` horizontally and `dy` vertically,
    /// with previous point 10ms before current (giving controllable speed).
    fn swiping_state(dx: f64, dy: f64, speed_px_per_ms: f64) -> SwipeState {
        // Place previous point so that speed = distance_prev_to_curr / 10ms
        let dist_prev_to_curr = speed_px_per_ms * 10.0;
        // previous is directly behind current along the dominant axis
        let (prev_x, prev_y) = if dx.abs() >= dy.abs() {
            (dx - dist_prev_to_curr.copysign(dx), dy)
        } else {
            (dx, dy - dist_prev_to_curr.copysign(dy))
        };
        SwipeState {
            start_point: Some(tp(0.0, 0.0, 0)),
            previous_point: Some(tp(prev_x, prev_y, 90)),
            current_point: Some(tp(dx, dy, 100)),
            ..SwipeState::default()
        }
    }

    #[test]
    fn test_swipe_left_over_distance() {
        // Move 111px left → exceeds distance_threshold(100)
        let mut state = swiping_state(-111.0, 0.0, 1.0);
        state.set_latest_swipe(&default_config());
        assert_eq!(state.latest_swipe, Some(Direction::Left));
    }

    #[test]
    fn test_swipe_right_over_distance() {
        let mut state = swiping_state(111.0, 0.0, 1.0);
        state.set_latest_swipe(&default_config());
        assert_eq!(state.latest_swipe, Some(Direction::Right));
    }

    #[test]
    fn test_swipe_up_over_distance() {
        let mut state = swiping_state(0.0, -111.0, 1.0);
        state.set_latest_swipe(&default_config());
        assert_eq!(state.latest_swipe, Some(Direction::Up));
    }

    #[test]
    fn test_swipe_down_over_distance() {
        let mut state = swiping_state(0.0, 111.0, 1.0);
        state.set_latest_swipe(&default_config());
        assert_eq!(state.latest_swipe, Some(Direction::Down));
    }

    #[test]
    fn test_swipe_not_triggered_below_threshold() {
        // dist=50 (< 100) and speed=1.0 (< 5.0) → no swipe
        let mut state = swiping_state(50.0, 0.0, 1.0);
        state.set_latest_swipe(&default_config());
        assert!(state.latest_swipe.is_none());
    }

    #[test]
    fn test_swipe_triggers_via_speed() {
        // dist=15 (> 10 minimum for speed to count), speed=6.0 (> 5.0 threshold)
        let mut state = swiping_state(15.0, 0.0, 6.0);
        state.set_latest_swipe(&default_config());
        assert_eq!(state.latest_swipe, Some(Direction::Right));
    }

    #[test]
    fn test_swipe_disallowed_direction_not_set() {
        // Move left far enough, but config only allows Right
        let config = SwipeConfig::new(vec![Direction::Right], 100.0, 5.0);
        let mut state = swiping_state(-111.0, 0.0, 1.0);
        state.set_latest_swipe(&config);
        assert!(state.latest_swipe.is_none());
    }

    // ── reset ─────────────────────────────────────────────────────────────────

    #[test]
    fn test_reset_clears_all_state() {
        let mut state = SwipeState {
            start_point: Some(tp(0.0, 0.0, 0)),
            previous_point: Some(tp(10.0, 0.0, 50)),
            current_point: Some(tp(20.0, 0.0, 100)),
            latest_swipe: Some(Direction::Right),
            traversing_axis: Some(Axis::X),
            is_swiping: true,
            return_animation_seconds: 0.5,
        };
        state.reset();
        assert!(state.start_point.is_none());
        assert!(state.previous_point.is_none());
        assert!(state.current_point.is_none());
        assert!(state.traversing_axis.is_none());
        assert!(!state.is_swiping);
        // latest_swipe and return_animation_seconds are intentionally NOT cleared by reset
    }
}
