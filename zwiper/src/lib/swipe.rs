use dioxus::prelude::*;

#[derive(Debug, Clone, PartialEq)]
pub enum Place {
    Home,
    Login,
    Register,
}

#[derive(Debug, Clone)]
pub struct State {
    pub place: Place,
    pub start: Option<(i64, i64)>,
    pub moving: bool,
    pub distance: f64,
}

impl State {
    pub fn new() -> Self {
        Self {
            place: Place::Home,
            moving: false,
            distance: 0.0,
            start: None,
        }
    }

    pub fn reset(&mut self) {
        self.moving = false;
        self.distance = 0.0;
        self.start = None;
    }
}
