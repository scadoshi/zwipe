use crate::inbound::ui::components::interactions::swipe::{
    config::SwipeConfig,
    direction::Direction as Dir,
    screen_offset::{ScreenOffset, ScreenOffsetMethods},
    state::SwipeState,
    Swipeable,
};
use dioxus::prelude::*;
use zwipe::domain::auth::models::session::Session;

#[component]
pub fn Profile(swipe_state: Signal<SwipeState>) -> Element {
    let swipe_config = SwipeConfig {
        navigation_swipes: vec![Dir::Up],
        submission_swipe: None,
        from_main_screen: ScreenOffset::down(),
    };

    let session: Signal<Option<Session>> = use_context();

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
                    button { class : "",
                        "logout"
                    }
                }
            }
        }
    }
}
