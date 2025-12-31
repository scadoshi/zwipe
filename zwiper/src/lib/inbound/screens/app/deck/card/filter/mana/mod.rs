mod color_identity_filter_mode;

use crate::inbound::components::{
    auth::bouncer::Bouncer,
    interactions::swipe::{config::SwipeConfig, state::SwipeState, Swipeable},
};
use color_identity_filter_mode::ColorIdentityFilterMode;
use dioxus::prelude::*;
use zwipe::domain::card::models::scryfall_data::colors::Color;
use zwipe::domain::card::models::search_card::card_filter::builder::CardFilterBuilder;

#[component]
pub fn Mana() -> Element {
    let swipe_config = SwipeConfig::blank();
    let swipe_state = use_signal(SwipeState::new);
    let navigator = use_navigator();

    let mut filter_builder: Signal<CardFilterBuilder> = use_context();

    let mut error = use_signal(|| None::<String>);

    let mut cmc_equals_string = use_signal(|| {
        filter_builder()
            .cmc_equals()
            .map(|v| v.to_string())
            .unwrap_or_default()
    });
    let mut try_parse_cmc_equals = move || {
        if cmc_equals_string().is_empty() {
            filter_builder.write().unset_cmc_equals();
            return;
        }
        if let Ok(n) = cmc_equals_string().parse::<f64>() {
            filter_builder.write().set_cmc_equals(n);
            cmc_equals_string.set(n.to_string());
        } else {
            error.set(Some("invalid input".to_string()));
        }
    };

    let mut cmc_range_min_string = use_signal(|| {
        filter_builder()
            .cmc_range()
            .map(|(min, _)| min.to_string())
            .unwrap_or_default()
    });
    let mut cmc_range_max_string = use_signal(|| {
        filter_builder()
            .cmc_range()
            .map(|(_, max)| max.to_string())
            .unwrap_or_default()
    });

    let mut selected_colors = use_signal(|| {
        if let Some(colors) = filter_builder().color_identity_equals() {
            colors.to_vec()
        } else if let Some(colors) = filter_builder().color_identity_within() {
            colors.to_vec()
        } else {
            Vec::new()
        }
    });
    let mut color_identity_filter_mode = use_signal(|| {
        if filter_builder().color_identity_equals().is_some() {
            ColorIdentityFilterMode::Exact
        } else if filter_builder().color_identity_within().is_some() {
            ColorIdentityFilterMode::Within
        } else {
            ColorIdentityFilterMode::default()
        }
    });

    let mut try_parse_cmc_range = move || {
        if cmc_range_min_string().is_empty() || cmc_range_max_string.is_empty() {
            filter_builder.write().unset_cmc_range();
            return;
        }
        if let (Ok(min), Ok(max)) = (
            cmc_range_min_string().parse::<f64>(),
            cmc_range_max_string().parse::<f64>(),
        ) {
            filter_builder.write().set_cmc_range((min, max));
            cmc_range_min_string.set(min.to_string());
            cmc_range_max_string.set(max.to_string());
        } else {
            error.set(Some("invalid input".to_string()));
        }
    };

    use_effect(move || {
        let colors = selected_colors();
        if colors.is_empty() {
            filter_builder.write().unset_color_identity_equals();
            filter_builder.write().unset_color_identity_within();
        } else {
            match color_identity_filter_mode() {
                ColorIdentityFilterMode::Exact => {
                    filter_builder.write().unset_color_identity_within();
                    filter_builder
                        .write()
                        .set_color_identity_equals(colors.into());
                }
                ColorIdentityFilterMode::Within => {
                    filter_builder.write().unset_color_identity_equals();
                    filter_builder
                        .write()
                        .set_color_identity_within(colors.into());
                }
            }
        }
    });

    rsx! {
        Bouncer {
            Swipeable { state: swipe_state, config: swipe_config,
                div { class : "container-sm",
                    h2 { class: "text-center mb-2 font-light tracking-wider", "mana filter" }

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

                        label { class: "label", "color identity" }

                        div { class: "flex flex-wrap gap-1 mb-2 flex-center",
                            for color in Color::all() {
                                div {
                                    class: if selected_colors().contains(&color) {
                                        "mana-box selected"
                                    } else {
                                        "mana-box"
                                    },
                                    onclick: move |_| {
                                        let mut colors = selected_colors();
                                        if colors.contains(&color) {
                                            colors.retain(|c| c != &color);
                                        } else {
                                            colors.push(color);
                                        }
                                        selected_colors.set(colors);
                                    },
                                    { color.to_string().to_lowercase() }
                                }
                            }
                        }

                        label { class: "label", "color identity filter mode" }
                        button {
                            class: "btn",
                            r#type: "button",
                            onclick: move |_| {
                                color_identity_filter_mode.set(color_identity_filter_mode().toggle());
                            },
                            { color_identity_filter_mode().to_string().to_lowercase() }
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
