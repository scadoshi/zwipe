//! Timestamped point for velocity calculations.

use chrono::NaiveDateTime;
use dioxus::html::geometry::ClientPoint;

/// A point with an associated timestamp for calculating swipe velocity.
#[derive(Debug, Clone)]
pub struct TimePoint {
    /// The screen coordinates of the point.
    pub point: ClientPoint,
    /// The timestamp when this point was recorded.
    pub time: NaiveDateTime,
}

impl TimePoint {
    /// Creates a new TimePoint with the given coordinates and timestamp.
    pub fn new(point: ClientPoint, time: NaiveDateTime) -> Self {
        Self { point, time }
    }
}
