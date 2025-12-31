use chrono::NaiveDateTime;
use dioxus::html::geometry::ClientPoint;

#[derive(Debug, Clone)]
pub struct TimePoint {
    pub point: ClientPoint,
    pub time: NaiveDateTime,
}

impl TimePoint {
    pub fn new(point: ClientPoint, time: NaiveDateTime) -> Self {
        Self { point, time }
    }
}
