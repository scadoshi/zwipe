use crate::{
    inbound::ui::components::interactions::swipe::{
        direction::Direction, onmouse::OnMouse, ontouch::OnTouch, state::SwipeState, VH_GAP,
    },
    outbound::client::auth::{session::ActiveSession, AuthClient},
};
use dioxus::prelude::*;
use std::time::Duration;
use tokio::time::interval;
use zwipe::domain::auth::models::session::Session;

#[component]
pub fn Profile(swipe_state: Signal<SwipeState>) -> Element {
    const MOVE_SWIPES: [Direction; 1] = [Direction::Up];

    let auth_client: Signal<AuthClient> = use_context();
    let mut session: Signal<Option<Session>> = use_context();

    let check_session = move || async move {
        let Some(s) = session.read().clone() else {
            return;
        };
        session.set(auth_client.read().infallible_get_active_session(&s).await);
    };

    use_effect(move || {
        spawn(async move {
            let mut i = interval(Duration::from_secs(60));
            loop {
                i.tick().await;
                check_session().await;
            }
        });
    });

    rsx! {
        if let Some(session) = session.read().as_ref() {
            div { class : "swipe-able",

                style : format!(
                    "transform: translateY(calc({}px - {}vh + {}vh));
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

                div { class : "profile-screen",
                    dl {
                        dt { "Username" }
                        dd { { session.user.username.to_string() } }
                        dt { "Email" }
                        dd { { session.user.email.to_string() } }
                    }
                }
            }
        }
    }
}
