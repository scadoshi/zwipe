use crate::inbound::ui::components::interactions::swipe::{
    delta::Delta, direction::Direction, time_point::TimePoint,
};
use chrono::Utc;
use dioxus::html::geometry::{
    euclid::{Point2D, UnknownUnit},
    ClientPoint,
};

type BasicPoint = Point2D<i32, UnknownUnit>;

#[derive(Debug, Clone)]
pub struct SwipeState {
    // for screen placement rendering
    pub start: Option<ClientPoint>,
    // for direction and speed calculation
    pub current: Option<TimePoint>,
    pub previous: Option<TimePoint>,
    // below should be used to
    // determine how quickly the element
    // returns to its swiped from position
    pub transition_seconds: f64,
    // what direction the last swipe resolved to
    pub previous_swipe: Option<Direction>,
    // determines screen displacement
    pub position: BasicPoint,
}

impl SwipeState {
    pub fn new() -> Self {
        Self {
            start: None,
            current: None,
            previous: None,
            transition_seconds: 0.0,
            previous_swipe: None,
            position: BasicPoint::new(0, 0),
        }
    }

    pub fn reset(&mut self) {
        self.start = None;
        self.current = None;
        self.previous = None;
    }

    pub fn dx(&self) -> Delta {
        if let (Some(start), Some(current), Some(previous)) = (
            self.start.as_ref(),
            self.current.as_ref(),
            self.previous.as_ref(),
        ) {
            let from_start = current.point.x - start.x;
            let from_previous = current.point.x - previous.point.x;

            let milliseconds_from_previous =
                (current.time - previous.time).as_seconds_f64() / 1000.0;
            let direction_from_start = if from_start > 0.0 {
                Direction::Right
            } else {
                Direction::Left
            };
            return Delta::new(
                from_start,
                from_previous,
                Some(direction_from_start),
                milliseconds_from_previous,
            );
        }
        Delta::default()
    }

    pub fn dy(&self) -> Delta {
        if let (Some(start), Some(current), Some(previous)) = (
            self.start.as_ref(),
            self.current.as_ref(),
            self.previous.as_ref(),
        ) {
            let from_start = current.point.y - start.y;
            let from_previous = current.point.y - previous.point.y;

            let milliseconds_from_previous =
                (current.time - previous.time).as_seconds_f64() / 1000.0;
            let direction_from_start = if from_start > 0.0 {
                Direction::Down
            } else {
                Direction::Up
            };
            return Delta::new(
                from_start,
                from_previous,
                Some(direction_from_start),
                milliseconds_from_previous,
            );
        }
        Delta::default()
    }

    pub fn set_transition_seconds(&mut self) {
        let mut s = 0.0;
        let md = self.dx().from_start.abs() + self.dy().from_start.abs();
        if md > 0.0 {
            s = 0.1;
        }
        if md > 25.0 {
            s = 0.2;
        }
        if md > 50.0 {
            s = 0.3;
        }
        if md > 100.0 {
            s = 0.4;
        }
        self.transition_seconds = s;
    }

    pub fn resolve_swipe_direction(&mut self, swipe_moves: &[Direction]) {
        const SPEED_CHECK_MIN_DIST: f64 = 10.0;
        const SPEED_THRESHOLD: f64 = 5.0;
        const DISTANCE_THRESHOLD: f64 = 200.0;

        if swipe_moves.is_empty() {
            return;
        }

        let dx = self.dx();
        let dy = self.dy();

        let mut x_dir = None;
        let mut y_dir = None;

        if dx.from_start.abs() > DISTANCE_THRESHOLD {
            x_dir = dx.direction_from_start.clone();
        }
        if dx.from_start.abs() > SPEED_CHECK_MIN_DIST {
            if let Some(speed) = dx.speed() {
                if speed.abs() > SPEED_THRESHOLD {
                    x_dir = dx.direction_from_start.clone();
                }
            }
        }

        if dy.from_start.abs() > DISTANCE_THRESHOLD {
            y_dir = dy.direction_from_start.clone();
        }
        if dy.from_start.abs() > SPEED_CHECK_MIN_DIST {
            if let Some(speed) = dy.speed() {
                if speed.abs() > SPEED_THRESHOLD {
                    y_dir = dy.direction_from_start.clone();
                }
            }
        }

        let allow = move |dir: &Direction| swipe_moves.contains(dir);
        match (x_dir, y_dir) {
            (Some(x), Some(y)) => {
                let winner = if dx.from_start.abs() > dy.from_start.abs() {
                    x
                } else {
                    y
                };
                if allow(&winner) {
                    match winner {
                        Direction::Up | Direction::Down => self.position.y += winner.as_i32(),
                        Direction::Left | Direction::Right => self.position.x += winner.as_i32(),
                    }
                }
                self.previous_swipe = Some(winner);
            }
            (Some(x), None) => {
                if allow(&x) {
                    self.position.x += x.as_i32();
                }
                self.previous_swipe = Some(x);
            }
            (None, Some(y)) => {
                if allow(&y) {
                    self.position.y += y.as_i32();
                }
                self.previous_swipe = Some(y);
            }
            (None, None) => self.previous_swipe = None,
        }
        tracing::debug!("swipe dir={:?}", self.previous_swipe);
    }

    // functionality shared by ontouch- and onmouse-
    pub fn onswipestart(&mut self, point: ClientPoint) {
        self.set_transition_seconds();
        let time_point = TimePoint::new(point.clone(), Utc::now().naive_utc());
        self.start = Some(point);
        self.current = Some(time_point.clone());
        self.previous = Some(time_point);
        tracing::debug!(
            "swipe start={:?}",
            self.current.as_ref().expect("failed to get swipe info")
        );
    }

    pub fn onswipemove(&mut self, point: ClientPoint) {
        let time_point = TimePoint::new(point, Utc::now().naive_utc());
        self.previous = self.current.clone();
        self.current = Some(time_point);
    }

    pub fn onswipeend(&mut self, point: ClientPoint, swipe_moves: &[Direction]) {
        self.set_transition_seconds();
        let time_point = TimePoint::new(point, Utc::now().naive_utc());
        self.previous = self.current.clone();
        self.current = Some(time_point);
        self.resolve_swipe_direction(swipe_moves);
        tracing::debug!(
            "swipe end={:?}",
            self.current.as_ref().expect("failed to get swipe info")
        );
        self.reset();
    }
}
