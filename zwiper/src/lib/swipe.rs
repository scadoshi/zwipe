use dioxus::{
    html::{geometry::ClientPoint, input_data::MouseButton},
    prelude::*,
};

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

pub fn handle_ontouchstart(e: Event<TouchData>, swipe_state: &mut Signal<State>) {
    if let Some(t) = e.touches().into_iter().next() {
        swipe_state.with_mut(|ss| {
            ss.start = Some(t.client_coordinates());
            ss.current = ss.start.clone();
        });

        println!("touch start={:?}", swipe_state.read().start);
    }
}

pub fn handle_ontouchmove(e: Event<TouchData>, swipe_state: &mut Signal<State>) {
    if let Some(t) = e.touches().into_iter().next() {
        swipe_state.with_mut(|ss| {
            ss.moving = true;
            ss.current = Some(t.client_coordinates());
            if let Some(start) = ss.start {
                if let Some(current) = ss.current {
                    ss.dx = start.x - current.x;
                    ss.dy = start.y - current.y;
                }
            }
        });
    }
}

pub fn handle_ontouchend(e: Event<TouchData>, swipe_state: &mut Signal<State>) {
    swipe_state.with_mut(|ss| {
        ss.moving = false;
        if let Some(t) = e.touches().into_iter().next() {
            ss.current = Some(t.client_coordinates());
        }
    });
    let ss = swipe_state.clone();
    println!("touch end={:?}", ss.read().current);

    swipe_state.with_mut(|ss| ss.reset());
}

pub fn handle_onmousedown(e: Event<MouseData>, swipe_state: &mut Signal<State>) {
    swipe_state.with_mut(|ss| {
        ss.start = Some(e.client_coordinates());
        ss.current = ss.start.clone();
    });

    println!("mouse down={:?}", swipe_state.read().start);
}

pub fn handle_onmousemove(e: Event<MouseData>, swipe_state: &mut Signal<State>) {
    if e.held_buttons().contains(MouseButton::Primary) {
        swipe_state.with_mut(|ss| {
            ss.moving = true;
            ss.current = Some(e.client_coordinates());
            if let Some(start) = ss.start {
                if let Some(current) = ss.current {
                    ss.dx = start.x - current.x;
                    ss.dy = start.y - current.y;
                }
            }
        });
    }
}

pub fn handle_onmouseup(e: Event<MouseData>, swipe_state: &mut Signal<State>) {
    swipe_state.with_mut(|ss| {
        ss.moving = false;
        ss.current = Some(e.client_coordinates());
    });
    let ss = swipe_state.clone();
    println!("mouse up={:?}", ss.read().current);

    swipe_state.with_mut(|ss| ss.reset());
}
