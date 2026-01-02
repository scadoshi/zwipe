mod color_identity_filter_mode;

use crate::inbound::components::auth::bouncer::Bouncer;
use color_identity_filter_mode::ColorIdentityFilterMode;
use dioxus::prelude::*;
use zwipe::domain::card::models::scryfall_data::colors::Color;
use zwipe::domain::card::models::search_card::card_filter::builder::CardFilterBuilder;

#[component]
pub fn Mana() -> Element {
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
            div { class: "fixed top-0 left-0 h-screen flex flex-col items-center overflow-y-auto",
                style: "width: 100vw; justify-content: center;",
                div { class : "container-sm",

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
                    }
                }
            }

            div { class: "util-bar",
                button {
                    class: "util-btn",
                    onclick: move |_| navigator.go_back(),
                    "back"
                }
            }
        }
    }
}

#[component]
pub fn ManaFilterContent() -> Element {
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
        if cmc_range_min_string().is_empty() || cmc_range_max_string().is_empty() {
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
        div { class: "flex-col gap-half",
            label { class: "label-xs", r#for: "cmc-equals", "cmc equals" }
            input { class: "input input-compact",
                id: "cmc-equals",
                placeholder: "cmc equals",
                value: cmc_equals_string(),
                r#type: "text",
                autocapitalize: "none",
                spellcheck: "false",
                oninput: move |event| {
                    error.set(None);
                    cmc_equals_string.set(event.value())
                },
                onblur: move |_| {
                    try_parse_cmc_equals();
                }
            }

            label { class: "label-xs", r#for: "cmc-range", "cmc range" }
            div { class: "flex-row mb-1",
                input { class: "input input-half-compact",
                    id: "cmc-range-min",
                    placeholder: "min",
                    value: cmc_range_min_string(),
                    r#type: "text",
                    autocapitalize: "none",
                    spellcheck: "false",
                    oninput: move |event| {
                        error.set(None);
                        cmc_range_min_string.set(event.value())
                    },
                    onblur: move |_| {
                        try_parse_cmc_range();
                    }
                }
                input { class: "input input-half-compact",
                    id: "cmc-range-max",
                    placeholder: "max",
                    value: cmc_range_max_string(),
                    r#type: "text",
                    autocapitalize: "none",
                    spellcheck: "false",
                    oninput: move |event| {
                        error.set(None);
                        cmc_range_max_string.set(event.value())
                    },
                    onblur: move |_| {
                        try_parse_cmc_range();
                    }
                }
            }

            label { class: "label-xs", "color identity" }

            div { class: "flex flex-wrap gap-1 mb-1 flex-center",
                for color in Color::all() {
                    div {
                        class: if selected_colors().contains(&color) {
                            "mana-box-compact selected"
                        } else {
                            "mana-box-compact"
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

            label { class: "label-xs", "filter mode" }
            div { class: "mb-1",
                button { class: "btn-xs",
                    r#type: "button",
                    onclick: move |_| {
                        color_identity_filter_mode.set(color_identity_filter_mode().toggle());
                    },
                    { color_identity_filter_mode().to_string().to_lowercase() }
                }
            }

            if let Some(error) = error() {
                div { class: "message-error", "{error}" }
            }
        }
    }
}
