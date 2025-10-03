use dioxus::html::geometry::ClientPoint;

#[derive(Debug, Clone, PartialEq)]
pub enum Place {
    Home,
    Login,
    Register,
}

#[derive(Debug, Clone)]
pub struct State {
    pub place: Place,
    pub start: Option<ClientPoint>,
    pub current: Option<ClientPoint>,
    pub dx: f64,
    pub dy: f64,
    pub moving: bool,
}

impl State {
    pub fn new() -> Self {
        Self {
            place: Place::Home,
            start: None,
            current: None,
            dx: 0.0,
            dy: 0.0,
            moving: false,
        }
    }

    pub fn reset(&mut self) {
        self.moving = false;
        self.start = None;
        self.current = None;
        self.dx = 0.0;
        self.dy = 0.0;
    }
}
