use dioxus::prelude::*;
use std::time::Duration;
use zwipe::domain::{auth::models::session::Session, logo::logo};

use crate::{
    inbound::ui::{
        components::{
            interactions::swipe::{
                config::SwipeConfig, direction::Direction as Dir, state::SwipeState, Swipeable,
            },
            screens::app::{decks::Decks, profile::Profile},
        },
        Router,
    },
    outbound::client::auth::{session::ActiveSession, AuthClient},
};

#[component]
pub fn Home() -> Element {
    let swipe_state = use_signal(|| SwipeState::new());
    let swipe_config = SwipeConfig {
        navigation_swipes: vec![Dir::Up, Dir::Down],
        submission_swipe: None,
        from_main_screen: None,
    };

    let auth_client: Signal<AuthClient> = use_context();
    let mut session: Signal<Option<Session>> = use_context();

    let check_session = move || async move {
        let Some(s) = session.read().clone() else {
            return;
        };
        session.set(auth_client.read().infallible_get_active_session(&s).await);
    };

    use_effect({
        let navigator = use_navigator();
        if session.read().is_none() {
            tracing::info!("session not found sending to auth home");
            navigator.push(Router::AuthHome {});
        }
        move || {
            spawn(async move {
                let mut interval = tokio::time::interval(Duration::from_secs(60));
                loop {
                    interval.tick().await;
                    check_session().await;
                }
            });
        }
    });

    let logo = logo();

    rsx! {
        Profile { swipe_state }
        Swipeable { state: swipe_state, config: swipe_config,
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
        Decks { swipe_state }
    }
}
