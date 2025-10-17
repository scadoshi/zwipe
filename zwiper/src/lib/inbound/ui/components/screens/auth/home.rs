use dioxus::prelude::*;
use zwipe::domain::{auth::models::session::Session, logo};

use crate::inbound::ui::components::{
    interactions::swipe::{
        direction::Direction, onswipe::OnMouse, ontouch::OnTouch, state::SwipeState, VH_GAP,
    },
    screens::auth::{login::Login, register::Register},
};

#[component]
pub fn Home() -> Element {
    const MOVE_SWIPES: [Direction; 2] = [Direction::Up, Direction::Down];

    let session: Signal<Option<Session>> = use_context();
    let mut swipe_state = use_signal(|| SwipeState::new());

    let logo = logo::logo();

    rsx! {
        if session.read().is_none() { Login {swipe_state} }

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
            ontouchend : move |e: Event<TouchData>| swipe_state.ontouchend(e, &MOVE_SWIPES),

            onmousedown : move |e: Event<MouseData>| swipe_state.onmousedown(e),
            onmousemove : move |e: Event<MouseData>| swipe_state.onmousemove(e),
            onmouseup : move |e: Event<MouseData>| swipe_state.onmouseup(e, &MOVE_SWIPES),

            div { class : "home-screen",

                div { class : "home-up-hint",
                    p { class : "up-arrow", "↑" },
                    p { "login" },
                },

                div { class: "logo",  "{logo}" },

                div { class : "home-down-hint",
                    p { "create profile" },
                    p { class : "down-arrow", "↓" },
                },
            }
        }

        if session.read().is_none() { Register {swipe_state} }
    }
}
