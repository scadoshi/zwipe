pub mod delta;
pub mod direction;
pub mod onswipe;
pub mod ontouch;
pub mod state;
pub mod time_point;

use crate::inbound::ui::components::interactions::swipe::{
    direction::Direction, onswipe::OnMouse, ontouch::OnTouch, state::SwipeState,
};
use dioxus::prelude::*;

pub const VH_GAP: i32 = 75;

#[component]
pub fn Swipeable(children: Element, swipe_dirs: Vec<Direction>) -> Element {
    let mut swipe_state = use_signal(|| SwipeState::new());
    let swipe_dirs1 = swipe_dirs.clone();
    let swipe_dirs2 = swipe_dirs.clone();

    rsx! {
            div { class : "swipeable",
                style : format!(
                    "transform: translateY(calc({}px + {}vh));
                    transition: transform {}s;",
                    swipe_state.read().dy().from_start,
                    swipe_state.read().position.y * VH_GAP,
                    swipe_state.read().transition_seconds
                ),

                ontouchstart : move |e: Event<TouchData>| swipe_state.ontouchstart(e),
                ontouchmove : move |e: Event<TouchData>| swipe_state.ontouchmove(e),
                ontouchend : move |e: Event<TouchData>| swipe_state.ontouchend(e, &swipe_dirs1),

                onmousedown : move |e: Event<MouseData>| swipe_state.onmousedown(e),
                onmousemove : move |e: Event<MouseData>| swipe_state.onmousemove(e),
                onmouseup : move |e: Event<MouseData>| swipe_state.onmouseup(e, &swipe_dirs2),
            }

        { children }
    }
}
