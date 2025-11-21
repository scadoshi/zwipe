use dioxus::prelude::*;
use zwipe::domain::card::models::search_card::SearchCards;

use crate::inbound::ui::components::{
    auth::bouncer::Bouncer,
    interactions::swipe::{config::SwipeConfig, state::SwipeState, Swipeable},
};

#[component]
pub fn Stats() -> Element {
    let swipe_config = SwipeConfig::blank();
    let swipe_state = use_signal(SwipeState::new);
    let navigator = use_navigator();

    let mut filter: Signal<SearchCards> = use_context();
    rsx! {
        Bouncer {
            Swipeable { state: swipe_state, config: swipe_config,
                div { class : "container-sm",
                    h2 { class: "text-center mb-2 font-light tracking-wider", "mana filters" }

                    form { class : "flex-col text-center",
                        label { class: "label", r#for : "name-contains", "name contains" }
                        input { class : "input",
                            id : "name-contains",
                            placeholder : "name contains",
                            value : if let Some(name) = filter.read().name_contains.as_deref() {
                                name
                            } else { "" },
                            r#type : "text",
                            autocapitalize : "none",
                            spellcheck : "false",
                            oninput : move |event| {
                                filter.write().name_contains = Some(event.value());
                                if filter.read().name_contains == Some("".to_string()) {
                                    filter.write().name_contains = None;
                                }
                            }
                        }

                        button { class : "btn",
                            onclick : move |_| {
                                tracing::error!("within the filter element filter is blank: {}", filter.read().is_blank());
                                navigator.go_back();
                            },
                            "back"
                        }
                    }
                }
            }
        }
    }
}
