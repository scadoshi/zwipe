use dioxus::prelude::*;
use std::time::Duration;
use zwipe::domain::{auth::models::session::Session, logo::logo};

use crate::{
    inbound::ui::{
        components::{
            interactions::swipe::{
                direction::Direction, onswipe::OnMouse, ontouch::OnTouch, state::SwipeState, VH_GAP,
            },
            screens::app::{decks::Decks, profile::Profile},
        },
        Screen,
    },
    outbound::client::auth::{session::ActiveSession, AuthClient},
};

#[component]
pub fn Home() -> Element {
    const MOVE_SWIPES: [Direction; 2] = [Direction::Up, Direction::Down];

    let auth_client: Signal<AuthClient> = use_context();
    let mut session: Signal<Option<Session>> = use_context();

    let navigator = use_navigator();
    if session.read().is_none() {
        navigator.push(Screen::AuthHome {});
    }

    let check_session = move || async move {
        let Some(s) = session.read().clone() else {
            return;
        };
        session.set(auth_client.read().infallible_get_active_session(&s).await);
    };

    use_effect(move || {
        spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60));
            loop {
                interval.tick().await;
                check_session().await;
            }
        });
    });

    let mut swipe_state = use_signal(|| SwipeState::new());

    let logo = logo();

    rsx! {
        Profile {swipe_state}

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
                    p { "profile" },
                },

                div { class: "logo",  "{logo}" },

                div { class : "home-down-hint",
                    p { "decks" },
                    p { class : "down-arrow", "↓" },
                },
            }
        }

        Decks {swipe_state}
    }
}
