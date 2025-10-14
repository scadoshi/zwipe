use std::time::Duration;

use crate::{
    client::auth::{session::EnsureActive, AuthClient},
    swipe::{self, Direction as Dir, OnMouse, OnTouch, VH_GAP},
};
use dioxus::prelude::*;
use zwipe::domain::auth::models::session::Session;

#[component]
pub fn Decks(swipe_state: Signal<swipe::State>) -> Element {
    const MOVE_SWIPES: [Dir; 1] = [Dir::Down];

    let auth_client: Signal<AuthClient> = use_context();
    let mut session: Signal<Option<Session>> = use_context();

    let check_session = move || {
        spawn(async move {
            let Some(s) = session.read().clone() else {
                return;
            };
            session.set(auth_client.read().infallible_ensure_active(&s).await);
        });
    };

    use_effect(move || {
        spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60));
            loop {
                interval.tick().await;
                check_session();
            }
        });
    });

    rsx! {
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
