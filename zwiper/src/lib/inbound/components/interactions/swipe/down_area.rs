//! A surface that answers a down swipe and nothing else.
//!
//! The empty end of a swipe stack has no card to drag, but a down swipe there
//! still undoes the last swipe, as it does on a card. [`SwipeDownArea`] wraps
//! whatever stands in for the card and calls `on_down` when a gesture over
//! it resolves to a down swipe, by the same thresholds the stack uses.

use dioxus::prelude::*;

use crate::inbound::components::interactions::swipe::config::SwipeConfig;
use crate::inbound::components::interactions::swipe::direction::Direction;
use crate::inbound::components::interactions::swipe::onmouse::OnMouse;
use crate::inbound::components::interactions::swipe::ontouch::OnTouch;
use crate::inbound::components::interactions::swipe::state::SwipeState;

/// `children` that call `on_down` on a down swipe, judged by `config`'s
/// distance and speed thresholds. Every other direction does nothing.
#[component]
pub fn SwipeDownArea(config: SwipeConfig, on_down: EventHandler<()>, children: Element) -> Element {
    let mut state = use_signal(SwipeState::new);
    // Takes the resolved swipe off the state and answers a down one.
    let mut settle = move || {
        let latest = state.write().latest_swipe.take();
        if latest == Some(Direction::Down) {
            on_down.call(());
        }
    };
    let touch_config = config.clone();
    rsx! {
        div {
            class: "swipe-down-area",
            // The gesture is ours, not the page's scroll.
            style: "touch-action: none;",
            ontouchstart: move |e: Event<TouchData>| state.ontouchstart(e),
            ontouchmove: move |e: Event<TouchData>| state.ontouchmove(e),
            ontouchend: move |e: Event<TouchData>| {
                state.ontouchend(e, &touch_config);
                settle();
            },
            onmousedown: move |e: Event<MouseData>| state.onmousedown(e),
            onmousemove: move |e: Event<MouseData>| state.onmousemove(e),
            onmouseup: move |e: Event<MouseData>| {
                state.onmouseup(e, &config);
                settle();
            },
            {children}
        }
    }
}
