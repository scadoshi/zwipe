//! Horizontal snap-to-page carousel.
//!
//! Provides a [`Carousel`] component that renders children in a horizontal
//! strip and handles touch/mouse gestures for page navigation with snap behavior.

/// Page indicator dots.
pub mod dots;
/// Carousel state and snap logic.
pub mod state;

use dioxus::{html::input_data::MouseButton, prelude::*};
use state::CarouselState;

/// A horizontal snap-to-page carousel.
///
/// Each direct child is rendered as one page at 100% width. The carousel
/// tracks drag gestures and snaps to the nearest page on release.
#[component]
pub fn Carousel(mut state: Signal<CarouselState>, children: Element) -> Element {
    let translate_x = state().translate_x_px();
    let transition_ms = state().snap_transition_ms;

    let transition_style = if transition_ms > 0 {
        format!("transform: translateX({translate_x}px); transition: transform {transition_ms}ms ease-out;")
    } else {
        format!("transform: translateX({translate_x}px); transition: none;")
    };

    rsx! {
        div {
            class: "carousel-viewport",

            // Touch events (mobile)
            ontouchstart: move |e| {
                if let Some(t) = e.touches().into_iter().next() {
                    state.with_mut(|s| s.on_drag_start(t.client_coordinates().x));
                }
            },
            ontouchmove: move |e| {
                if let Some(t) = e.touches().into_iter().next() {
                    state.with_mut(|s| s.on_drag_move(t.client_coordinates().x));
                }
            },
            ontouchend: move |e| {
                if let Some(t) = e.touches_changed().into_iter().next() {
                    state.with_mut(|s| s.on_drag_end(t.client_coordinates().x));
                }
            },

            // Mouse events (desktop)
            onmousedown: move |e| {
                state.with_mut(|s| s.on_drag_start(e.client_coordinates().x));
            },
            onmousemove: move |e| {
                if e.held_buttons().contains(MouseButton::Primary) {
                    state.with_mut(|s| s.on_drag_move(e.client_coordinates().x));
                }
            },
            onmouseup: move |e| {
                state.with_mut(|s| s.on_drag_end(e.client_coordinates().x));
            },

            div {
                class: "carousel-strip",
                style: "{transition_style}",
                { children }
            }
        }
    }
}
