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

#[derive(Debug, Clone)]
pub struct State {
    // screen state
    pub screen: Screen,
    // point (coordinate) data
    pub start: Option<ClientPoint>,
    pub previous: Option<ClientPoint>,
    pub current: Option<ClientPoint>,
    // delta (change) in x or y
    pub dx: f64,
    pub dy: f64,
    // timestamp of when
    // x or y reverses direction
    pub latest_x_pivot: Option<NaiveDateTime>,
    pub latest_y_pivot: Option<NaiveDateTime>,
    // should be used to
    // determine how quickly the element
    // returns to its swiped from position
    pub transition_seconds: f64,
}

impl State {
    pub fn new() -> Self {
        Self {
            screen: Screen::Home,
            start: None,
            previous: None,
            current: None,
            dx: 0.0,
            dy: 0.0,
            latest_x_pivot: None,
            latest_y_pivot: None,
            transition_seconds: 0.0,
        }
    }

    pub fn reset(&mut self) {
        self.start = None;
        self.previous = None;
        self.current = None;
        self.dx = 0.0;
        self.dy = 0.0;
        self.latest_x_pivot = None;
        self.latest_y_pivot = None;
    }

    pub fn dx(&self) -> f64 {
        if let (Some(previous), Some(current)) = (self.start, self.current) {
            return previous.x - current.x;
        }
        return 0.0;
    }

    pub fn dy(&self) -> f64 {
        if let (Some(previous), Some(current)) = (self.start, self.current) {
            return previous.y - current.y;
        }
        return 0.0;
    }

    pub fn set_transition_seconds(&mut self) {
        let mut s = 0.0;
        let md = self.dx().abs() + self.dy().abs();
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

    pub fn vx(&self) -> f64 {
        if let Some(lp) = self.latest_x_pivot {
            let milliseconds_passed = (Utc::now().naive_utc() - lp).as_seconds_f64() / 1000.0;

            if milliseconds_passed < 0.001 {
                return 0.0;
            }

            return (self.dx() / milliseconds_passed).abs();
        } else {
            return 0.0;
        }
    }

    pub fn vy(&self) -> f64 {
        if let Some(lp) = self.latest_y_pivot {
            let milliseconds_passed = (Utc::now().naive_utc() - lp).as_seconds_f64() / 1000.0;

            if milliseconds_passed < 0.001 {
                return 0.0;
            }

            return (self.dy() / milliseconds_passed).abs();
        } else {
            return 0.0;
        }
    }

    pub fn resolve_direction(&self, allowed: &[Direction]) -> Option<Direction> {
        const SPEED_THRESHOLD: f64 = 3.0;
        const DISTANCE_THRESHOLD: f64 = 50.0;

        if allowed.is_empty() {
            return None;
        }

        let allow = move |dir: &Direction| allowed.contains(dir);

        let (dx, vx) = (self.dx(), self.vx());
        let (dy, vy) = (self.dy(), self.vy());

        let mut x_dir = None;

        if dx.abs() > DISTANCE_THRESHOLD || vx > SPEED_THRESHOLD {
            if dx < 0.0 {
                x_dir = Some(Direction::Left);
            } else {
                x_dir = Some(Direction::Right);
            }
        }

        let mut y_dir = None;

        if dy.abs() > DISTANCE_THRESHOLD || vy > SPEED_THRESHOLD {
            if dy < 0.0 {
                y_dir = Some(Direction::Down);
            } else {
                y_dir = Some(Direction::Up);
            }
        }

        match (x_dir, y_dir) {
            (Some(x), Some(y)) => {
                if allow(&x) && (dx > dy || !allow(&y)) {
                    return Some(x);
                } else if allow(&y) && (dy > dx || !allow(&x)) {
                    return Some(y);
                }
                None
            }

            (Some(x), None) => {
                if allow(&x) {
                    return Some(x);
                } else {
                    return None;
                }
            }

            (None, Some(y)) => {
                if allow(&y) {
                    return Some(y);
                } else {
                    return None;
                }
            }

            (None, None) => None,
        }
    }
}

pub trait OnTouch {
    fn ontouchstart(&mut self, e: Event<TouchData>);
    fn ontouchmove(&mut self, e: Event<TouchData>);
    fn ontouchend(&mut self, e: Event<TouchData>);
}

impl OnTouch for Signal<State> {
    fn ontouchstart(&mut self, e: Event<TouchData>) {
        if let Some(t) = e.touches().into_iter().next() {
            self.with_mut(|ss| {
                ss.set_transition_seconds();
                ss.latest_x_pivot = Some(Utc::now().naive_utc());
                ss.latest_y_pivot = Some(Utc::now().naive_utc());

                let some_point = Some(t.client_coordinates());
                ss.start = some_point.clone();
                ss.previous = some_point.clone();
                ss.current = some_point;
            });

            println!("touch start={:?}", self.read().start);
        }
    }

    fn ontouchmove(&mut self, e: Event<TouchData>) {
        if let Some(t) = e.touches().into_iter().next() {
            self.with_mut(|ss| {
                ss.current = Some(t.client_coordinates());

                let new_dx = ss.dx();
                let new_dy = ss.dy();

                if new_dx.abs() < ss.dx.abs() {
                    ss.latest_x_pivot = Some(Utc::now().naive_utc());
                }

                if new_dy.abs() < ss.dy.abs() {
                    ss.latest_y_pivot = Some(Utc::now().naive_utc());
                }

                ss.dx = new_dx;
                ss.dy = new_dy;
            });
        }
    }

    fn ontouchend(&mut self, e: Event<TouchData>) {
        self.with_mut(|ss| {
            ss.set_transition_seconds();
            if let Some(t) = e.touches().into_iter().next() {
                ss.current = Some(t.client_coordinates());
            }
        });
        let ss = self.clone();
        println!("touch end={:?}", ss.read().current);

        self.with_mut(|ss| ss.reset());
    }
}

pub trait OnMouse {
    fn onmousedown(&mut self, e: Event<MouseData>);
    fn onmousemove(&mut self, e: Event<MouseData>);
    fn onmouseup(&mut self, e: Event<MouseData>);
}

impl OnMouse for Signal<State> {
    fn onmousedown(&mut self, e: Event<MouseData>) {
        self.with_mut(|ss| {
            ss.set_transition_seconds();

            let some_point = Some(e.client_coordinates());
            ss.start = some_point.clone();
            ss.current = some_point.clone();
            ss.previous = some_point;
        });

        println!("touch start={:?}", self.read().start);
    }

    fn onmousemove(&mut self, e: Event<MouseData>) {
        if e.held_buttons().contains(MouseButton::Primary) {
            self.with_mut(|ss| {
                ss.current = Some(e.client_coordinates());
            });
        }
    }

    fn onmouseup(&mut self, e: Event<MouseData>) {
        self.with_mut(|ss| {
            ss.set_transition_seconds();
            ss.current = Some(e.client_coordinates());
        });
        let ss = self.clone();
        println!("touch end={:?}", ss.read().current);

        self.with_mut(|ss| ss.reset());
    }
}
