use crate::{
    inbound::ui::components::{
        interactions::swipe::{
            config::SwipeConfig, direction::Direction as Dir, screen_offset::ScreenOffset,
            state::SwipeState, Swipeable,
        },
        screens::app::{decks::Decks, profile::Profile},
    },
    outbound::client::auth::{session::ActiveSession, AuthClient},
};
use dioxus::prelude::*;
use std::time::Duration;
use tokio::time::interval;
use zwipe::domain::{auth::models::session::Session, logo::logo};

#[component]
pub fn Home() -> Element {
    let swipe_state = use_signal(|| SwipeState::new());
    let swipe_config = SwipeConfig {
        navigation_swipes: vec![Dir::Up, Dir::Down],
        submission_swipe: None,
        from_main_screen: ScreenOffset::new(0, 0),
    };

    let auth_client: Signal<AuthClient> = use_context();
    let mut session: Signal<Option<Session>> = use_context();

    spawn(async move {
        let mut interval = interval(Duration::from_secs(60));
        loop {
            tracing::debug!("ensuring active session");
            interval.tick().await;
            let Some(s) = session.read().clone() else {
                continue;
            };
            session.set(auth_client.read().infallible_get_active_session(&s).await);
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
