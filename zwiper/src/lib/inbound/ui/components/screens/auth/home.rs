use dioxus::prelude::*;
use zwipe::domain::logo;

use crate::inbound::ui::components::{
    interactions::swipe::{
        config::SwipeConfig, direction::Direction as Dir, state::SwipeState, Swipeable,
    },
    screens::auth::{login::Login, register::Register},
};

#[component]
pub fn Home() -> Element {
    let swipe_state = use_signal(|| SwipeState::new());
    let swipe_config = SwipeConfig {
        navigation_swipes: vec![Dir::Up, Dir::Down],
        submission_swipe: None,
        from_main_screen: None,
    };

    let logo = logo::logo();

    // No navigation effects needed - RouteGuard handles screen switching

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
