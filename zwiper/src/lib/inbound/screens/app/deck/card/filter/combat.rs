use dioxus::prelude::*;
use zwipe::domain::card::models::search_card::card_filter::builder::CardFilterBuilder;

use crate::inbound::components::{
    auth::bouncer::Bouncer,
    interactions::swipe::{config::SwipeConfig, state::SwipeState, Swipeable},
};

#[component]
pub fn Combat() -> Element {
    let swipe_config = SwipeConfig::blank();
    let swipe_state = use_signal(SwipeState::new);
    let navigator = use_navigator();

    let mut filter_builder: Signal<CardFilterBuilder> = use_context();

    let mut error = use_signal(|| None::<String>);

    // Power signals
    let mut power_equals_string = use_signal(String::new);
    let mut power_range_min_string = use_signal(String::new);
    let mut power_range_max_string = use_signal(String::new);

    // Toughness signals
    let mut toughness_equals_string = use_signal(String::new);
    let mut toughness_range_min_string = use_signal(String::new);
    let mut toughness_range_max_string = use_signal(String::new);

    // Power equals parser
    let mut try_parse_power_equals = move || {
        if power_equals_string().is_empty() {
            filter_builder.write().unset_power_equals();
            return;
        }
        if let Ok(n) = power_equals_string().parse::<i32>() {
            filter_builder.write().set_power_equals(n);
            power_equals_string.set(n.to_string());
        } else {
            error.set(Some("invalid input".to_string()));
        }
    };

    // Power range parser
    let mut try_parse_power_range = move || {
        if power_range_min_string().is_empty() || power_range_max_string().is_empty() {
            filter_builder.write().unset_power_range();
            return;
        }
        if let (Ok(min), Ok(max)) = (
            power_range_min_string().parse::<i32>(),
            power_range_max_string().parse::<i32>(),
        ) {
            filter_builder.write().set_power_range((min, max));
            power_range_min_string.set(min.to_string());
            power_range_max_string.set(max.to_string());
        } else {
            error.set(Some("invalid input".to_string()));
        }
    };

    // Toughness equals parser
    let mut try_parse_toughness_equals = move || {
        if toughness_equals_string().is_empty() {
            filter_builder.write().unset_toughness_equals();
            return;
        }
        if let Ok(n) = toughness_equals_string().parse::<i32>() {
            filter_builder.write().set_toughness_equals(n);
            toughness_equals_string.set(n.to_string());
        } else {
            error.set(Some("invalid input".to_string()));
        }
    };

    // Toughness range parser
    let mut try_parse_toughness_range = move || {
        if toughness_range_min_string().is_empty() || toughness_range_max_string().is_empty() {
            filter_builder.write().unset_toughness_range();
            return;
        }
        if let (Ok(min), Ok(max)) = (
            toughness_range_min_string().parse::<i32>(),
            toughness_range_max_string().parse::<i32>(),
        ) {
            filter_builder.write().set_toughness_range((min, max));
            toughness_range_min_string.set(min.to_string());
            toughness_range_max_string.set(max.to_string());
        } else {
            error.set(Some("invalid input".to_string()));
        }
    };

    rsx! {
        Bouncer {
            Swipeable { state: swipe_state, config: swipe_config,
                div { class : "container-sm",
                    h2 { class: "text-center mb-2 font-light tracking-wider", "combat filter" }
                    form { class : "flex-col text-center",
                        label { class: "label", r#for: "power-equals", "power equals" }
                        input { class: "input",
                            id: "power-equals",
                            placeholder: "power equals",
                            value: power_equals_string(),
                            r#type: "text",
                            autocapitalize: "none",
                            spellcheck: "false",
                            oninput: move |event| {
                                error.set(None);
                                power_equals_string.set(event.value())
                            },
                            onblur: move |_| {
                                try_parse_power_equals();
                            }
                        }

                        label { class: "label", r#for: "power-range", "power range" }
                        div { class: "flex-row mb-4",
                            input { class: "input input-half",
                                id: "power-range-min",
                                placeholder: "min",
                                value: power_range_min_string(),
                                r#type: "text",
                                autocapitalize: "none",
                                spellcheck: "false",
                                oninput: move |event| {
                                    error.set(None);
                                    power_range_min_string.set(event.value())
                                },
                                onblur: move |_| {
                                    try_parse_power_range();
                                }
                            }
                            input { class: "input input-half",
                                id: "power-range-max",
                                placeholder: "max",
                                value: power_range_max_string(),
                                r#type: "text",
                                autocapitalize: "none",
                                spellcheck: "false",
                                oninput: move |event| {
                                    error.set(None);
                                    power_range_max_string.set(event.value())
                                },
                                onblur: move |_| {
                                    try_parse_power_range();
                                }
                            }
                        }

                        label { class: "label", r#for: "toughness-equals", "toughness equals" }
                        input { class: "input",
                            id: "toughness-equals",
                            placeholder: "toughness equals",
                            value: toughness_equals_string(),
                            r#type: "text",
                            autocapitalize: "none",
                            spellcheck: "false",
                            oninput: move |event| {
                                error.set(None);
                                toughness_equals_string.set(event.value())
                            },
                            onblur: move |_| {
                                try_parse_toughness_equals();
                            }
                        }

                        label { class: "label", r#for: "toughness-range", "toughness range" }
                        div { class: "flex-row mb-4",
                            input { class: "input input-half",
                                id: "toughness-range-min",
                                placeholder: "min",
                                value: toughness_range_min_string(),
                                r#type: "text",
                                autocapitalize: "none",
                                spellcheck: "false",
                                oninput: move |event| {
                                    error.set(None);
                                    toughness_range_min_string.set(event.value())
                                },
                                onblur: move |_| {
                                    try_parse_toughness_range();
                                }
                            }
                            input { class: "input input-half",
                                id: "toughness-range-max",
                                placeholder: "max",
                                value: toughness_range_max_string(),
                                r#type: "text",
                                autocapitalize: "none",
                                spellcheck: "false",
                                oninput: move |event| {
                                    error.set(None);
                                    toughness_range_max_string.set(event.value())
                                },
                                onblur: move |_| {
                                    try_parse_toughness_range();
                                }
                            }
                        }

                        if let Some(error) = error() {
                            div { class: "message-error", "{error}" }
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
