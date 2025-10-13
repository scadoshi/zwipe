use crate::swipe::{self, Direction as Dir, OnMouse, OnTouch, VH_GAP};
use dioxus::prelude::*;
use zwipe::domain::auth::models::session::Session;

#[component]
pub fn Decks(swipe_state: Signal<swipe::State>) -> Element {
    const MOVE_SWIPES: [Dir; 1] = [Dir::Down];
    let session: Signal<Option<Session>> = use_context();

    rsx! {
        if let Some(session) = session.read().as_ref() {
            div { class : "swipe-able",

                style : format!(
                    "transform: translateY(calc({}px + {}vh + {}vh));
                    transition: transform {}s;",
                    swipe_state.read().dy().from_start,
                    VH_GAP,
                    swipe_state.read().position.y * VH_GAP,
                    swipe_state.read().transition_seconds
                ),

                ontouchstart : move |e: Event<TouchData>| swipe_state.ontouchstart(e),
                ontouchmove : move |e: Event<TouchData>| swipe_state.ontouchmove(e),
                ontouchend : move |e: Event<TouchData>| { swipe_state.ontouchend(e, &MOVE_SWIPES) },

                onmousedown : move |e: Event<MouseData>| swipe_state.onmousedown(e),
                onmousemove : move |e: Event<MouseData>| swipe_state.onmousemove(e),
                onmouseup : move |e: Event<MouseData>| { swipe_state.onmouseup(e, &MOVE_SWIPES) },

                div { class : "decks-screen",
                    p {
                        "under construction"
                    }
                }
            }
        }
    }
}
