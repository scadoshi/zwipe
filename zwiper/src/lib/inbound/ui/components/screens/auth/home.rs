use dioxus::prelude::*;
use zwipe::domain::{auth::models::session::Session, logo};

use crate::inbound::ui::{
    components::{
        interactions::swipe::{
            config::SwipeConfig, direction::Direction as Dir, state::SwipeState, Swipeable,
        },
        screens::auth::{login::Login, register::Register},
    },
    Router,
};

#[component]
pub fn Home() -> Element {
    let swipe_state = use_signal(|| SwipeState::new());
    let swipe_config = SwipeConfig {
        navigation_swipes: vec![Dir::Up, Dir::Down],
        submission_swipe: None,
        from_main_screen: None,
    };

    let session: Signal<Option<Session>> = use_context();
    let navigator = use_navigator();
    if session.read().is_some() {
        tracing::info!("session found sending to app home");
        navigator.push(Router::AppHome {});
    }

    let logo = logo::logo();

    use_effect(move || {
        let navigator = use_navigator();
        if session.read().is_none() {
            tracing::info!("session not found sending to auth home");
            navigator.push(Router::AuthHome {});
        }
    });

    rsx! {
        Login { swipe_state }
        Swipeable { state: swipe_state, config: swipe_config,
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
        Register { swipe_state }
    }
}
