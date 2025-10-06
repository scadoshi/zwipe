use chrono::{NaiveDateTime, Utc};
use dioxus::{
    html::{
        geometry::{
            euclid::{Point2D, UnknownUnit},
            ClientPoint,
        },
        input_data::MouseButton,
    },
    prelude::*,
};

pub const VH_GAP: i32 = 75;
type BasicPoint = Point2D<i32, UnknownUnit>;

#[derive(Debug, Clone, PartialEq)]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}

impl Direction {
    pub fn as_i32(&self) -> i32 {
        match self {
            Dir::Left | Dir::Up => -1,
            Dir::Right | Dir::Down => 1,
        }
    }
}

use Direction as Dir;

#[derive(Debug, Clone)]
pub struct Delta {
    pub from_start: f64,
    pub direction_from_start: Option<Dir>,
    pub from_previous: f64,
    pub milliseconds_from_previous: f64,
}

impl Default for Delta {
    fn default() -> Self {
        Self {
            from_start: 0.0,
            direction_from_start: None,
            from_previous: 0.0,
            milliseconds_from_previous: 0.0,
        }
    }
}

impl Delta {
    pub fn new(
        from_start: f64,
        from_previous: f64,
        direction_from_start: Option<Dir>,
        milliseconds_from_previous: f64,
    ) -> Self {
        Self {
            from_start,
            from_previous,
            direction_from_start,
            milliseconds_from_previous,
        }
    }

    pub fn speed(&self) -> Option<f64> {
        if self.milliseconds_from_previous < 0.001 {
            return None;
        }
        Some(self.from_previous / self.milliseconds_from_previous)
    }
}

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

#[derive(Debug, Clone)]
pub struct State {
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
    pub previous_swipe: Option<Dir>,
    // determines screen displacement
    pub position: BasicPoint,
}

impl State {
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
                Dir::Right
            } else {
                Dir::Left
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
            let direction_from_start = if from_start > 0.0 { Dir::Down } else { Dir::Up };
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

    pub fn resolve_swipe_direction(&mut self, swipe_moves: &[Dir]) {
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

        let allow = move |dir: &Dir| swipe_moves.contains(dir);
        match (x_dir, y_dir) {
            (Some(x), Some(y)) => {
                let winner = if dx.from_start.abs() > dy.from_start.abs() {
                    x
                } else {
                    y
                };
                if allow(&winner) {
                    match winner {
                        Dir::Up | Dir::Down => self.position.y += winner.as_i32(),
                        Dir::Left | Dir::Right => self.position.x += winner.as_i32(),
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
        println!("swipe dir => {:?}", self.previous_swipe);
    }

    // functionality shared by ontouch- and onmouse-
    pub fn onswipestart(&mut self, point: ClientPoint) {
        self.set_transition_seconds();
        let time_point = TimePoint::new(point.clone(), Utc::now().naive_utc());
        self.start = Some(point);
        self.current = Some(time_point.clone());
        self.previous = Some(time_point);
        println!(
            "swipe start => {:?}",
            self.current
                .as_ref()
                .expect("failed to get swipe start timepoint")
        );
    }

    pub fn onswipemove(&mut self, point: ClientPoint) {
        let time_point = TimePoint::new(point, Utc::now().naive_utc());
        self.previous = self.current.clone();
        self.current = Some(time_point);
    }

    pub fn onswipeend(&mut self, point: ClientPoint, swipe_moves: &[Dir]) {
        self.set_transition_seconds();
        let time_point = TimePoint::new(point, Utc::now().naive_utc());
        self.previous = self.current.clone();
        self.current = Some(time_point);
        self.resolve_swipe_direction(swipe_moves);
        println!(
            "swipe end => {:?}",
            self.current
                .as_ref()
                .expect("failed to get swipe end timepoint")
        );
        self.reset();
    }
}

// wiring event for touch interactions to swipe behavior
pub trait OnTouch {
    fn ontouchstart(&mut self, e: Event<TouchData>);
    fn ontouchmove(&mut self, e: Event<TouchData>);
    fn ontouchend(&mut self, e: Event<TouchData>, swipe_moves: &[Dir]);
}

impl OnTouch for Signal<State> {
    fn ontouchstart(&mut self, e: Event<TouchData>) {
        if let Some(t) = e.touches().into_iter().next() {
            let point = t.client_coordinates();
            self.with_mut(|ss| ss.onswipestart(point));
        }
    }

    fn ontouchmove(&mut self, e: Event<TouchData>) {
        if let Some(t) = e.touches().into_iter().next() {
            let point = t.client_coordinates();
            self.with_mut(|ss| ss.onswipemove(point));
        }
    }

    fn ontouchend(&mut self, e: Event<TouchData>, swipe_moves: &[Dir]) {
        if let Some(t) = e.touches_changed().into_iter().next() {
            let point = t.client_coordinates();
            self.with_mut(|ss| ss.onswipeend(point, swipe_moves));
        }
    }
}

// wiring event for mouse interactions to swipe behavior
// only used while developing on desktop emulation
pub trait OnMouse {
    fn onmousedown(&mut self, e: Event<MouseData>);
    fn onmousemove(&mut self, e: Event<MouseData>);
    fn onmouseup(&mut self, e: Event<MouseData>, swipe_moves: &[Dir]);
}

impl OnMouse for Signal<State> {
    fn onmousedown(&mut self, e: Event<MouseData>) {
        let point = e.client_coordinates();
        self.with_mut(|ss| ss.onswipestart(point));
    }

    fn onmousemove(&mut self, e: Event<MouseData>) {
        if e.held_buttons().contains(MouseButton::Primary) {
            let point = e.client_coordinates();
            self.with_mut(|ss| ss.onswipemove(point));
        }
    }

    fn onmouseup(&mut self, e: Event<MouseData>, swipe_moves: &[Dir]) {
        let point = e.client_coordinates();
        self.with_mut(|ss| ss.onswipeend(point, swipe_moves));
    }
}
