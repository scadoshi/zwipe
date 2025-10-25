use crate::inbound::ui::components::interactions::swipe::{config::SwipeConfig, state::SwipeState};
use dioxus::prelude::*;
use uuid::Uuid;

#[component]
pub fn GetDeck(deck_id: Uuid) -> Element {
    let swipe_state = use_signal(|| SwipeState::new());
    let swipe_config = SwipeConfig::blank();

    let navigator = use_navigator();

    rsx! {
        p { "{deck_id" }
    }
}
