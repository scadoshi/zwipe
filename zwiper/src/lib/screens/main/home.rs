use dioxus::prelude::*;
use zwipe::domain::{auth::models::session::Session, logo::logo};

use crate::{
    screens::Screen,
    swipe::{self, Direction as Dir, OnMouse, OnTouch, VH_GAP},
};

#[component]
pub fn Home() -> Element {
    const MOVE_SWIPES: [Dir; 2] = [Dir::Up, Dir::Down];

    let session: Signal<Option<Session>> = use_context();

    let navigator = use_navigator();
    let mut swipe_state = use_signal(|| swipe::State::new());

    let logo = logo();

    rsx! {

        if session.read().is_none() {
            { navigator.push(Screen::AuthHome {}); }
        }

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
                    p { "decks" },
                },

                div { class: "logo",  "{logo}" },

                div { class : "home-down-hint",
                    p { "profile" },
                    p { class : "down-arrow", "↓" },
                },
            }
        }
    }
}
