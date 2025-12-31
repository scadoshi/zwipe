use crate::inbound::components::interactions::swipe::{
    config::SwipeConfig, onswipe::OnSwipe, state::SwipeState,
};
use dioxus::prelude::*;

// wiring event for touch interactions to swipe behavior
pub trait OnTouch {
    fn ontouchstart(&mut self, e: Event<TouchData>);
    fn ontouchmove(&mut self, e: Event<TouchData>);
    fn ontouchend(&mut self, e: Event<TouchData>, config: &SwipeConfig);
}

impl OnTouch for Signal<SwipeState> {
    fn ontouchstart(&mut self, e: Event<TouchData>) {
        if let Some(t) = e.touches().into_iter().next() {
            let start_point = t.client_coordinates();
            self.with_mut(|ss| ss.onswipestart(start_point));
        }
    }

    fn ontouchmove(&mut self, e: Event<TouchData>) {
        if let Some(t) = e.touches().into_iter().next() {
            let current_point = t.client_coordinates();
            self.with_mut(|ss| ss.onswipemove(current_point));
        }
    }

    fn ontouchend(&mut self, e: Event<TouchData>, config: &SwipeConfig) {
        if let Some(t) = e.touches_changed().into_iter().next() {
            let end_point = t.client_coordinates();
            self.with_mut(|ss| ss.onswipeend(end_point, config));
        }
    }
}
