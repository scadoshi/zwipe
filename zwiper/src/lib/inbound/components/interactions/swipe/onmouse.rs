use crate::inbound::components::interactions::swipe::{
    config::SwipeConfig, onswipe::OnSwipe, state::SwipeState,
};
use dioxus::{html::input_data::MouseButton, prelude::*};

// wiring event for mouse interactions to swipe behavior
// only used while developing on desktop emulation
pub trait OnMouse {
    fn onmousedown(&mut self, e: Event<MouseData>);
    fn onmousemove(&mut self, e: Event<MouseData>);
    fn onmouseup(&mut self, e: Event<MouseData>, config: &SwipeConfig);
}

impl OnMouse for Signal<SwipeState> {
    fn onmousedown(&mut self, e: Event<MouseData>) {
        let start_point = e.client_coordinates();
        self.with_mut(|ss| ss.onswipestart(start_point));
    }

    fn onmousemove(&mut self, e: Event<MouseData>) {
        if e.held_buttons().contains(MouseButton::Primary) {
            let current_point = e.client_coordinates();
            self.with_mut(|ss| ss.onswipemove(current_point));
        }
    }

    fn onmouseup(&mut self, e: Event<MouseData>, config: &SwipeConfig) {
        let end_point = e.client_coordinates();
        self.with_mut(|ss| ss.onswipeend(end_point, config));
    }
}
