use dioxus::prelude::*;

#[derive(Debug, Clone, PartialEq)]
pub enum SwipePosition {
    Login,
    Register,
}

#[derive(Debug, Clone)]
pub struct SwipeState {
    pub screen: SwipePosition,
    pub is_dragging: bool,
    pub drag_offset: f64,
    pub start_x: Option<f64>,
}

impl SwipeState {
    pub fn new() -> Self {
        Self {
            screen: SwipePosition::Login,
            is_dragging: false,
            drag_offset: 0.0,
            start_x: None,
        }
    }

    pub fn reset(&mut self) {
        self.is_dragging = false;
        self.drag_offset = 0.0;
        self.start_x = None;
    }
}
