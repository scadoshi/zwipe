use crate::inbound::ui::components::interactions::swipe::{
    axis::Axis, config::SwipeConfig, direction::Direction as Dir, screen_offset::ScreenOffset,
    time_point::TimePoint,
};
use chrono::Utc;
use dioxus::html::geometry::{
    euclid::{Point2D, UnknownUnit},
    ClientPoint,
};

type DeltaPoint = Point2D<f64, UnknownUnit>;

#[derive(Debug, Clone)]
pub struct SwipeState {
    // for screen placement rendering
    pub start_point: Option<TimePoint>,
    // for direction and speed calculation
    pub previous_point: Option<TimePoint>,
    pub current_point: Option<TimePoint>,
    // what direction the last swipe resolved to
    pub latest_swipe: Option<Dir>,
    pub screen_offset: ScreenOffset,
    // tracks which axis user is swiping on
    pub traversing_axis: Option<Axis>,
    pub is_swiping: bool,
    // how quickly element returns back to start position
    pub return_animation_seconds: f64,
}

impl SwipeState {
    pub fn new() -> Self {
        Self {
            start_point: None,
            current_point: None,
            previous_point: None,
            latest_swipe: None,
            screen_offset: ScreenOffset::new(0, 0),
            traversing_axis: None,
            is_swiping: false,
            return_animation_seconds: 0.0,
        }
    }

    pub fn reset(&mut self) {
        self.start_point = None;
        self.current_point = None;
        self.previous_point = None;
        self.traversing_axis = None;
        self.is_swiping = false;
    }

    pub fn distance_from_start_point(&self) -> Option<f64> {
        let (Some(start), Some(curr)) = (self.start_point.as_ref(), self.current_point.as_ref())
        else {
            return None;
        };

        let dx = curr.point.x - start.point.x;
        let dy = curr.point.y - start.point.y;

        Some((dx.powi(2) + dy.powi(2)).sqrt())
    }

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

    pub fn distance_from_previous_point(&self) -> Option<f64> {
        let (Some(prev), Some(curr)) = (self.previous_point.as_ref(), self.current_point.as_ref())
        else {
            return None;
        };

        let dx = curr.point.x - prev.point.x;
        let dy = curr.point.y - prev.point.y;

        Some((dx.powi(2) + dy.powi(2)).sqrt())
    }

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

    pub fn milliseconds_from_previous_point(&self) -> Option<f64> {
        let (Some(prev), Some(curr)) = (self.previous_point.as_ref(), self.current_point.as_ref())
        else {
            return None;
        };
        Some((curr.time - prev.time).num_milliseconds() as f64)
    }

    pub fn speed(&self) -> Option<f64> {
        let (Some(distance), Some(time)) = (
            self.distance_from_previous_point(),
            self.milliseconds_from_previous_point(),
        ) else {
            return None;
        };
        Some(distance / time)
    }

    // below should be used to
    // determine how quickly the element
    // returns to its swiped from position
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

    pub fn update_position(&mut self, direction: &Dir) {
        match direction {
            Dir::Left => self.screen_offset.x -= 1,
            Dir::Right => self.screen_offset.x += 1,
            Dir::Up => self.screen_offset.y -= 1,
            Dir::Down => self.screen_offset.y += 1,
        }
    }

    pub fn set_latest_swipe(&mut self, config: &SwipeConfig) {
        const DISTANCE_THRESHOLD_FOR_SPEED_TO_BE_VALID: f64 = 10.0;
        const SPEED_THRESHOLD_FOR_SWIPE: f64 = 5.0;
        const DISTANCE_THRESHOLD_FOR_SWIPE: f64 = 100.0;

        if self.traversing_axis.is_none() {
            self.set_traversing_axis();
        }

        let (Some(distance_from_start), Some(speed)) =
            (self.distance_from_start_point(), self.speed())
        else {
            return;
        };

        if distance_from_start > DISTANCE_THRESHOLD_FOR_SWIPE
            || (distance_from_start > DISTANCE_THRESHOLD_FOR_SPEED_TO_BE_VALID
                && speed > SPEED_THRESHOLD_FOR_SWIPE)
        {
            match self.traversing_axis {
                Some(Axis::X) => {
                    let Some(DeltaPoint { x, .. }) = self.delta_from_start_point() else {
                        return;
                    };

                    if x < 0.0 {
                        let direction = Dir::Left;
                        if config.navigation_swipes.contains(&direction) {
                            self.update_position(&direction);
                        }
                        self.latest_swipe = Some(direction);
                    }
                    if x > 0.0 {
                        let direction = Dir::Right;
                        if config.navigation_swipes.contains(&direction) {
                            self.update_position(&direction);
                        }
                        self.latest_swipe = Some(direction);
                    }
                }

                Some(Axis::Y) => {
                    let Some(DeltaPoint { y, .. }) = self.delta_from_start_point() else {
                        return;
                    };

                    if y < 0.0 {
                        let direction = Dir::Up;
                        if config.navigation_swipes.contains(&direction) {
                            self.update_position(&direction);
                        }
                        self.latest_swipe = Some(direction);
                    }
                    if y > 0.0 {
                        let direction = Dir::Down;
                        if config.navigation_swipes.contains(&direction) {
                            self.update_position(&direction);
                        }
                        self.latest_swipe = Some(direction);
                    }
                }

                None => (),
            }
        }

        tracing::trace!("swipe dir={:?}", self.latest_swipe);
    }

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
