use crate::inbound::ui::components::{
    auth::bouncer::Bouncer,
    interactions::swipe::{config::SwipeConfig, state::SwipeState, Swipeable},
};
use dioxus::{html::tr, prelude::*};
use zwipe::domain::card::models::search_card::SearchCards;

#[component]
pub fn Mana() -> Element {
    let swipe_config = SwipeConfig::blank();
    let swipe_state = use_signal(|| SwipeState::new());
    let navigator = use_navigator();

    let mut filter: Signal<SearchCards> = use_context();

    let mut error = use_signal(|| None::<String>);
    /*
       ✅ pub cmc_equals: Option<f64>,
       ✅ pub cmc_range: Option<(f64, f64)>,
       ⚠️ pub color_identity_equals: Option<Colors>,
       ⚠️ pub color_identity_contains_any: Option<Colors>,
    */

    let mut cmc_equals_string = use_signal(|| String::new());
    let mut try_parse_cmc_equals = move || {
        if cmc_equals_string().is_empty() {
            filter.write().cmc_equals = None;
            return;
        }
        if let Ok(n) = cmc_equals_string().parse::<f64>() {
            filter.write().cmc_equals = Some(n);
            cmc_equals_string.set(n.to_string());
        } else {
            error.set(Some("invalid input".to_string()));
        }
    };

    let mut cmc_range_min_string = use_signal(|| String::new());
    let mut cmc_range_max_string = use_signal(|| String::new());
    let mut try_parse_cmc_range = move || {
        if cmc_range_min_string().is_empty() || cmc_range_max_string.is_empty() {
            filter.write().cmc_range = None;
            tracing::error!("don't forget the other one!");
            return;
        }
        if let (Ok(min), Ok(max)) = (
            cmc_range_min_string().parse::<f64>(),
            cmc_range_max_string().parse::<f64>(),
        ) {
            filter.write().cmc_range = Some((min, max));
            cmc_range_min_string.set(min.to_string());
            cmc_range_max_string.set(max.to_string());
        } else {
            error.set(Some("invalid input".to_string()));
        }
    };

    rsx! {
        Bouncer {
            Swipeable { state: swipe_state, config: swipe_config,
                div { class : "container-sm",
                    h2 { class: "text-center mb-2 font-light tracking-wider", "mana filters" }

                    form { class : "flex-col text-center",

                        label { class: "label", r#for : "cmc-equals", "cmc equals" }
                        input { class : "input",
                            id : "cmc-equals",
                            placeholder : "cmc equals",
                            value : cmc_equals_string(),
                            r#type : "text",
                            autocapitalize : "none",
                            spellcheck : "false",
                            oninput : move |event| {
                                error.set(None);
                                cmc_equals_string.set(event.value())
                            },
                            onblur : move |_| {
                                try_parse_cmc_equals();
                            }
                        }

                        label { class: "label", r#for : "cmc-range", "cmc range" }
                        div { class : "flex-row mb-4",
                            input { class : "input input-half",
                                id : "cmc-range",
                                placeholder : "min",
                                value : cmc_range_min_string(),
                                r#type : "text",
                                autocapitalize : "none",
                                spellcheck : "false",
                                oninput : move |event| {
                                    error.set(None);
                                    cmc_range_min_string.set(event.value())
                                },
                                onblur : move |_| {
                                    try_parse_cmc_range();
                                }
                            }
                            input { class : "input input-half",
                                id : "cmc-range",
                                placeholder : "max",
                                value : cmc_range_max_string(),
                                r#type : "text",
                                autocapitalize : "none",
                                spellcheck : "false",
                                oninput : move |event| {
                                    error.set(None);
                                    cmc_range_max_string.set(event.value())
                                },
                                onblur : move |_| {
                                    try_parse_cmc_range();
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
