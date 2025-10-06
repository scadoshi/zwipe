use dioxus::prelude::*;
use zwipe::domain::ascii_logo;

use crate::{
    screens::auth::{login::Login, register::Register},
    swipe::{self, Direction as Dir, OnMouse, OnTouch, VH_GAP},
};

#[component]
pub fn Home() -> Element {
    let ascii_logo = ascii_logo::logo();
    let mut swipe_state = use_signal(|| swipe::State::new());

    rsx! {
        Login {swipe_state}

        div { class : "swipe-able",

            style : format!(
                "transform: translateY(calc({}px + {}vh));
                transition: transform {}s;",
                swipe_state.read().dy().from_start,
                swipe_state.read().position.y * VH_GAP,
                swipe_state.read().transition_seconds
            ),

            ontouchstart : move |e: Event<TouchData>| swipe_state.ontouchstart(e),
            ontouchmove : move |e: Event<TouchData>| swipe_state.ontouchmove(e),
            ontouchend : move |e: Event<TouchData>| swipe_state.ontouchend(e, &[Dir::Up, Dir::Down]),

            onmousedown : move |e: Event<MouseData>| swipe_state.onmousedown(e),
            onmousemove : move |e: Event<MouseData>| swipe_state.onmousemove(e),
            onmouseup : move |e: Event<MouseData>| swipe_state.onmouseup(e, &[Dir::Up, Dir::Down]),

            div { class : "home-screen",

                div { class : "home-up-hint",
                    p { class : "up-arrow", "↑" },
                    p { "login" },
                },

                pre { class: "ascii-logo", "{ascii_logo}" },

                div { class : "home-down-hint",
                    p { "create profile" },
                    p { class : "down-arrow", "↓" },
                },
            }
        }

        Register {swipe_state}
    }
}
