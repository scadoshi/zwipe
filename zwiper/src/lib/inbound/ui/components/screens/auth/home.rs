use dioxus::prelude::*;
use zwipe::domain::{auth::models::session::Session, logo};

use crate::inbound::ui::components::{
    interactions::swipe::{
        config::SwipeConfig, direction::Direction, onmouse::OnMouse, ontouch::OnTouch,
        state::SwipeState, Swipeable, VH_GAP,
    },
    screens::auth::{login::Login, register::Register},
};

#[component]
pub fn Home() -> Element {
    let swipe_state = use_signal(|| SwipeState::new());
    let swipe_config: SwipeConfig =
        SwipeConfig::new(vec![Direction::Up, Direction::Down], None, None);

    let logo = logo::logo();

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
