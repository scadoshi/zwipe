use crate::inbound::ui::components::interactions::swipe::{
    direction::Direction, state::SwipeState,
};
use dioxus::prelude::*;

// wiring event for touch interactions to swipe behavior
pub trait OnTouch {
    fn ontouchstart(&mut self, e: Event<TouchData>);
    fn ontouchmove(&mut self, e: Event<TouchData>);
    fn ontouchend(&mut self, e: Event<TouchData>, swipe_dirs: &[Direction]);
}

impl OnTouch for Signal<SwipeState> {
    fn ontouchstart(&mut self, e: Event<TouchData>) {
        if let Some(t) = e.touches().into_iter().next() {
            let point = t.client_coordinates();
            self.with_mut(|ss| ss.onswipestart(point));
        }
    }

    fn ontouchmove(&mut self, e: Event<TouchData>) {
        if let Some(t) = e.touches().into_iter().next() {
            let point = t.client_coordinates();
            self.with_mut(|ss| ss.onswipemove(point));
        }
    }

    fn ontouchend(&mut self, e: Event<TouchData>, swipe_dirs: &[Direction]) {
        if let Some(t) = e.touches_changed().into_iter().next() {
            let point = t.client_coordinates();
            self.with_mut(|ss| ss.onswipeend(point, swipe_dirs));
        }
    }
}
