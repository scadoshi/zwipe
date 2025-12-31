use crate::inbound::components::{
    auth::bouncer::Bouncer,
    interactions::swipe::{config::SwipeConfig, state::SwipeState, Swipeable},
};
use dioxus::prelude::*;
use zwipe::domain::card::models::search_card::card_filter::builder::CardFilterBuilder;

#[component]
pub fn Text() -> Element {
    let swipe_config = SwipeConfig::blank();
    let swipe_state = use_signal(SwipeState::new);
    let navigator = use_navigator();

    let mut filter_builder: Signal<CardFilterBuilder> = use_context();
    rsx! {
        Bouncer {
            Swipeable { state: swipe_state, config: swipe_config,
                div { class : "container-sm",
                    h2 { class: "text-center mb-2 font-light tracking-wider", "text filter" }

                    form { class : "flex-col text-center",
                        label { class: "label", r#for : "name-contains", "name contains" }
                        input { class : "input",
                            id : "name-contains",
                            placeholder : "name contains",
                            value : if let Some(name) = filter_builder().name_contains() {
                                name
                            } else { "" },
                            r#type : "text",
                            autocapitalize : "none",
                            spellcheck : "false",
                            oninput : move |event| {
                                filter_builder.write().set_name_contains(event.value());
                            }
                        }

                        label { class: "label", r#for: "oracle-text-contains", "oracle text contains" }
                        input { class: "input",
                            id: "oracle-text-contains",
                            placeholder: "oracle text contains",
                            value: if let Some(text) = filter_builder().oracle_text_contains() {
                                text
                            } else { "" },
                            r#type: "text",
                            autocapitalize: "none",
                            spellcheck: "false",
                            oninput: move |event| {
                                filter_builder.write().set_oracle_text_contains(event.value());
                            }
                        }

                        button { class : "btn",
                            onclick : move |_| {
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
