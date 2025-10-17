use crate::inbound::ui::components::interactions::swipe::{
    direction::Direction, state::SwipeState,
};
use dioxus::{html::input_data::MouseButton, prelude::*};

// wiring event for mouse interactions to swipe behavior
// only used while developing on desktop emulation
pub trait OnMouse {
    fn onmousedown(&mut self, e: Event<MouseData>);
    fn onmousemove(&mut self, e: Event<MouseData>);
    fn onmouseup(&mut self, e: Event<MouseData>, swipe_moves: &[Direction]);
}

impl OnMouse for Signal<SwipeState> {
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

    fn onmouseup(&mut self, e: Event<MouseData>, swipe_moves: &[Direction]) {
        let point = e.client_coordinates();
        self.with_mut(|ss| ss.onswipeend(point, swipe_moves));
    }
}
