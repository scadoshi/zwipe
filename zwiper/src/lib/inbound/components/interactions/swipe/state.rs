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
