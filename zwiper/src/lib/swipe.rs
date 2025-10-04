use chrono::{NaiveDateTime, Utc};
use dioxus::{
    html::{geometry::ClientPoint, input_data::MouseButton},
    prelude::*,
};

#[derive(Debug, Clone, PartialEq)]
pub enum Screen {
    Home,
    Login,
    Register,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}
use Direction as Dir;

#[derive(Debug, Clone)]
pub struct Delta {
    pub from_start: f64,
    pub from_previous: f64,
    pub direction: Option<Dir>,
    pub milliseconds: f64,
}

impl Default for Delta {
    fn default() -> Self {
        Self {
            from_start: 0.0,
            from_previous: 0.0,
            direction: None,
            milliseconds: 0.0,
        }
    }
}

impl Delta {
    pub fn new(
        from_start: f64,
        from_previous: f64,
        direction: Option<Dir>,
        milliseconds: f64,
    ) -> Self {
        Self {
            from_start,
            from_previous,
            direction,
            milliseconds,
        }
    }

    pub fn speed(&self) -> Option<f64> {
        if self.milliseconds < 0.001 {
            return None;
        }
        Some(self.from_previous / self.milliseconds)
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
    // screen state
    pub screen: Screen,
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
}

impl State {
    pub fn new() -> Self {
        Self {
            screen: Screen::Home,
            start: None,
            current: None,
            previous: None,
            transition_seconds: 0.0,
            previous_swipe: None,
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

            let milliseconds = (current.time - previous.time).as_seconds_f64() / 1000.0;
            let direction = if from_start > 0.0 {
                Dir::Right
            } else {
                Dir::Left
            };
            return Delta::new(from_start, from_previous, Some(direction), milliseconds);
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

            let milliseconds = (current.time - previous.time).as_seconds_f64() / 1000.0;
            let direction = if from_start > 0.0 { Dir::Down } else { Dir::Up };
            return Delta::new(from_start, from_previous, Some(direction), milliseconds);
        }
        Delta::default()
    }

    pub fn set_transition_seconds(&mut self) {
        let mut s = 0.0;
        let md = self.dx().from_start.abs() + self.dy().from_start.abs();
        if md > 0.0 {
            s = 0.2;
        }
        if md > 25.0 {
            s = 0.3;
        }
        if md > 50.0 {
            s = 0.4;
        }
        if md > 100.0 {
            s = 0.5;
        }
        self.transition_seconds = s;
    }

    pub fn set_direction(&mut self, allowed: &[Dir]) {
        const SPEED_THRESHOLD: f64 = 3.0;
        const DISTANCE_THRESHOLD: f64 = 50.0;

        if allowed.is_empty() {
            return;
        }

        let dx = self.dx();
        let dy = self.dy();

        let mut x_dir = None;
        let mut y_dir = None;

        if dx.from_start.abs() > DISTANCE_THRESHOLD {
            x_dir = dx.direction.clone();
        }
        if let Some(speed) = dx.speed() {
            if speed.abs() > SPEED_THRESHOLD {
                x_dir = dx.direction.clone();
            }
        }

        if dy.from_start.abs() > DISTANCE_THRESHOLD {
            y_dir = dy.direction.clone();
        }
        if let Some(speed) = dy.speed() {
            if speed.abs() > SPEED_THRESHOLD {
                y_dir = dy.direction.clone();
            }
        }

        let allow = move |dir: &Dir| allowed.contains(dir);
        match (x_dir, y_dir) {
            (Some(x), Some(y)) => {
                if allow(&x) && (dx.from_start.abs() > dy.from_start.abs() || !allow(&y)) {
                    self.previous_swipe = Some(x);
                } else if allow(&y) && (dy.from_start.abs() > dx.from_start.abs() || !allow(&x)) {
                    self.previous_swipe = Some(y);
                }
            }
            (Some(x), None) => {
                if allow(&x) {
                    self.previous_swipe = Some(x);
                }
            }
            (None, Some(y)) => {
                if allow(&y) {
                    self.previous_swipe = Some(y);
                }
            }
            (None, None) => (),
        }
    }
}

pub trait OnTouch {
    fn ontouchstart(&mut self, e: Event<TouchData>);
    fn ontouchmove(&mut self, e: Event<TouchData>);
    fn ontouchend(&mut self, e: Event<TouchData>, allowed: &[Dir]);
}

impl OnTouch for Signal<State> {
    fn ontouchstart(&mut self, e: Event<TouchData>) {
        if let Some(t) = e.touches().into_iter().next() {
            self.with_mut(|ss| {
                ss.set_transition_seconds();

                let point = t.client_coordinates();
                let time_point = TimePoint::new(point.clone(), Utc::now().naive_utc());

                ss.start = Some(point);
                ss.current = Some(time_point.clone());
                ss.previous = Some(time_point);
            });
            println!(
                "touchstart => {:?}",
                self.read()
                    .current
                    .as_ref()
                    .expect("failed to get touchstart timepoint")
            );
        }
    }

    fn ontouchmove(&mut self, e: Event<TouchData>) {
        if let Some(t) = e.touches().into_iter().next() {
            self.with_mut(|ss| {
                let time_point = TimePoint::new(t.client_coordinates(), Utc::now().naive_utc());
                ss.previous = ss.current.clone();
                ss.current = Some(time_point);
            });
        }
    }

    fn ontouchend(&mut self, e: Event<TouchData>, allowed: &[Dir]) {
        if let Some(t) = e.touches_changed().into_iter().next() {
            self.with_mut(|ss| {
                ss.set_transition_seconds();

                let time_point = TimePoint::new(t.client_coordinates(), Utc::now().naive_utc());

                ss.previous = ss.current.clone();
                ss.current = Some(time_point);

                ss.set_direction(allowed);
            });
            println!(
                "touchend => {:?}",
                self.read()
                    .current
                    .as_ref()
                    .expect("failed to get touchend timepoint")
            );
            self.with_mut(|ss| ss.reset());
        }
    }
}

pub trait OnMouse {
    fn onmousedown(&mut self, e: Event<MouseData>);
    fn onmousemove(&mut self, e: Event<MouseData>);
    fn onmouseup(&mut self, e: Event<MouseData>, allowed: &[Dir]);
}

impl OnMouse for Signal<State> {
    fn onmousedown(&mut self, e: Event<MouseData>) {
        self.with_mut(|ss| {
            ss.set_transition_seconds();

            let point = e.client_coordinates();
            let time_point = TimePoint::new(point.clone(), Utc::now().naive_utc());

            ss.start = Some(point);
            ss.current = Some(time_point.clone());
            ss.previous = Some(time_point);
        });
        println!(
            "mousedown => {:?}",
            self.read()
                .current
                .as_ref()
                .expect("failed to get mousedown timepoint")
        );
    }

    fn onmousemove(&mut self, e: Event<MouseData>) {
        if e.held_buttons().contains(MouseButton::Primary) {
            self.with_mut(|ss| {
                let time_point = TimePoint::new(e.client_coordinates(), Utc::now().naive_utc());
                ss.previous = ss.current.clone();
                ss.current = Some(time_point);
            });
        }
    }

    fn onmouseup(&mut self, e: Event<MouseData>, allowed: &[Dir]) {
        self.with_mut(|ss| {
            ss.set_transition_seconds();

            let time_point = TimePoint::new(e.client_coordinates(), Utc::now().naive_utc());

            ss.previous = ss.current.clone();
            ss.current = Some(time_point);

            ss.set_direction(allowed);
        });
        println!(
            "mouseup => {:?}",
            self.read()
                .current
                .as_ref()
                .expect("failed to get mouseup timepoint")
        );
        self.with_mut(|ss| ss.reset());
    }
}
