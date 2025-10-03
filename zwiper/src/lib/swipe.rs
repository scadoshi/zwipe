use chrono::NaiveDateTime;
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

#[derive(Debug, Clone)]
pub struct State {
    pub screen: Screen,
    pub start: Option<ClientPoint>,
    pub previous: Option<ClientPoint>,
    pub current: Option<ClientPoint>,
    pub timestamp: Option<NaiveDateTime>,
    pub transition_seconds: f64,
}

impl State {
    pub fn new() -> Self {
        Self {
            screen: Screen::Home,
            start: None,
            previous: None,
            current: None,
            timestamp: None,
            transition_seconds: 0.0,
        }
    }

    pub fn reset(&mut self) {
        self.start = None;
        self.previous = None;
        self.current = None;
        self.timestamp = None;
    }

    pub fn dx(&self) -> f64 {
        if let (Some(start), Some(current)) = (self.start, self.current) {
            return start.x - current.x;
        }
        return 0.0;
    }

    pub fn dy(&self) -> f64 {
        if let (Some(start), Some(current)) = (self.start, self.current) {
            return start.y - current.y;
        }
        return 0.0;
    }

    pub fn speedy(&self) -> f64 {
        let now = chrono::Utc::now().naive_utc();
        todo!()
    }

    pub fn speedx(&self) -> f64 {
        let now = chrono::Utc::now().naive_utc();
        todo!()
    }
}

pub trait OnTouch {
    fn handle_ontouchstart(&mut self, e: Event<TouchData>);
    fn handle_ontouchmove(&mut self, e: Event<TouchData>);
    fn handle_ontouchend(&mut self, e: Event<TouchData>);
}

impl OnTouch for Signal<State> {
    fn handle_ontouchstart(&mut self, e: Event<TouchData>) {
        if let Some(t) = e.touches().into_iter().next() {
            self.with_mut(|ss| {
                ss.transition_seconds = 0.0;

                ss.start = Some(t.client_coordinates());
                ss.current = ss.start.clone();
            });

            println!("touch start={:?}", self.read().start);
        }
    }

    fn handle_ontouchmove(&mut self, e: Event<TouchData>) {
        if let Some(t) = e.touches().into_iter().next() {
            self.with_mut(|ss| {
                ss.current = Some(t.client_coordinates());
            });
        }
    }

    fn handle_ontouchend(&mut self, e: Event<TouchData>) {
        self.with_mut(|ss| {
            ss.transition_seconds = 0.5;
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
    fn handle_onmousedown(&mut self, e: Event<MouseData>);
    fn handle_onmousemove(&mut self, e: Event<MouseData>);
    fn handle_onmouseup(&mut self, e: Event<MouseData>);
}

impl OnMouse for Signal<State> {
    fn handle_onmousedown(&mut self, e: Event<MouseData>) {
        self.with_mut(|ss| {
            ss.transition_seconds = 0.0;

            let some_point = Some(e.client_coordinates());
            ss.start = some_point.clone();
            ss.current = some_point.clone();
            ss.previous = some_point.clone();
        });

        println!("touch start={:?}", self.read().start);
    }

    fn handle_onmousemove(&mut self, e: Event<MouseData>) {
        if e.held_buttons().contains(MouseButton::Primary) {
            self.with_mut(|ss| {
                ss.current = Some(e.client_coordinates());
            });
        }
    }

    fn handle_onmouseup(&mut self, e: Event<MouseData>) {
        self.with_mut(|ss| {
            ss.current = Some(e.client_coordinates());
        });
        let ss = self.clone();
        println!("touch end={:?}", ss.read().current);

        self.with_mut(|ss| ss.reset());
    }
}
