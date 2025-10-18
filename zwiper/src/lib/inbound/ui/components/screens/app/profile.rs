use crate::{
    inbound::ui::components::interactions::swipe::{
        config::SwipeConfig, direction::Direction as Dir, state::SwipeState, Swipeable,
    },
    outbound::client::auth::{session::ActiveSession, AuthClient},
};
use dioxus::prelude::*;
use std::time::Duration;
use tokio::time::interval;
use zwipe::domain::auth::models::session::Session;

#[component]
pub fn Profile(swipe_state: Signal<SwipeState>) -> Element {
    let swipe_config = SwipeConfig {
        navigation_swipes: vec![Dir::Up],
        submission_swipe: None,
        from_main_screen: Some(Dir::Up),
    };
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
            let mut interval = interval(Duration::from_secs(60));
            loop {
                interval.tick().await;
                check_session().await;
            }
        });
    });

    rsx! {
        if let Some(session) = session.read().as_ref() {
            Swipeable { state: swipe_state, config: swipe_config,
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
