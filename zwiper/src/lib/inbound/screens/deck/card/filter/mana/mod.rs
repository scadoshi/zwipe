use dioxus::prelude::*;
use zwipe::domain::card::models::scryfall_data::colors::Color;
use zwipe::domain::card::models::search_card::card_filter::builder::CardFilterBuilder;

use super::filter_mode::FilterMode;

#[component]
pub fn Mana() -> Element {
    let mut filter_builder: Signal<CardFilterBuilder> = use_context();

    let mut error = use_signal(|| None::<String>);

    // CMC mode signal
    let mut cmc_mode = use_signal(|| {
        if filter_builder().cmc_range().is_some() {
            FilterMode::Range
        } else {
            FilterMode::Exact
        }
    });

    // CMC string signals for input buffering (needed for validation)
    let mut cmc_equals_string = use_signal(|| {
        filter_builder()
            .cmc_equals()
            .map(|v| v.to_string())
            .unwrap_or_default()
    });
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

    // Parse and write CMC equals on blur
    let mut try_parse_cmc_equals = move || {
        if cmc_equals_string().is_empty() {
            filter_builder.write().unset_cmc_equals();
            error.set(None);
            return;
        }
        if let Ok(n) = cmc_equals_string().parse::<f64>() {
            filter_builder.write().set_cmc_equals(n);
            cmc_equals_string.set(n.to_string());
            error.set(None);
        } else {
            error.set(Some("invalid cmc".to_string()));
        }
    };

    // Parse and write CMC range on blur
    let mut try_parse_cmc_range = move || {
        if cmc_range_min_string().is_empty() && cmc_range_max_string().is_empty() {
            filter_builder.write().unset_cmc_range();
            error.set(None);
            return;
        }
        // Need both values for a valid range
        if cmc_range_min_string().is_empty() || cmc_range_max_string().is_empty() {
            // Don't write partial range, wait for both
            return;
        }
        if let (Ok(min), Ok(max)) = (
            cmc_range_min_string().parse::<f64>(),
            cmc_range_max_string().parse::<f64>(),
        ) {
            filter_builder.write().set_cmc_range((min, max));
            cmc_range_min_string.set(min.to_string());
            cmc_range_max_string.set(max.to_string());
            error.set(None);
        } else {
            error.set(Some("invalid cmc range".to_string()));
        }
    };

    // Color identity mode signal
    let mut color_identity_mode = use_signal(|| {
        if filter_builder().color_identity_equals().is_some() {
            FilterMode::Exact
        } else if filter_builder().color_identity_within().is_some() {
            FilterMode::Range
        } else {
            FilterMode::default()
        }
    });

    // Check if CMC filter is active (read directly from filter_builder)
    let cmc_is_active =
        filter_builder().cmc_equals().is_some() || filter_builder().cmc_range().is_some();

    // Check if color identity filter is active
    let color_is_active = filter_builder().color_identity_equals().is_some()
        || filter_builder().color_identity_within().is_some();

    // Get current selected colors from filter_builder
    let selected_colors = if let Some(colors) = filter_builder().color_identity_equals() {
        colors.to_vec()
    } else if let Some(colors) = filter_builder().color_identity_within() {
        colors.to_vec()
    } else {
        Vec::new()
    };

    rsx! {
        div { class: "flex-col gap-half",
            // CMC filter
            div { class: "label-row",
                label { class: "label-xs", "cmc" }
                button {
                    class: "clear-btn",
                    onclick: move |_| {
                        let new_mode = cmc_mode().toggle();
                        cmc_mode.set(new_mode);
                        // Clear the other mode's value when switching
                        let mut fb = filter_builder.write();
                        match new_mode {
                            FilterMode::Exact => {
                                fb.unset_cmc_range();
                                cmc_range_min_string.set(String::new());
                                cmc_range_max_string.set(String::new());
                            }
                            FilterMode::Range => {
                                fb.unset_cmc_equals();
                                cmc_equals_string.set(String::new());
                            }
                        }
                    },
                    "{cmc_mode().to_string().to_lowercase()}"
                }
                if cmc_is_active {
                    button {
                        class: "clear-btn",
                        onclick: move |_| {
                            let mut fb = filter_builder.write();
                            fb.unset_cmc_equals();
                            fb.unset_cmc_range();
                            cmc_equals_string.set(String::new());
                            cmc_range_min_string.set(String::new());
                            cmc_range_max_string.set(String::new());
                            error.set(None);
                        },
                        "×"
                    }
                }
            }

            match cmc_mode() {
                FilterMode::Exact => rsx! {
                    input { class: "input input-compact input-narrow",
                        id: "cmc-equals",
                        placeholder: "cmc",
                        value: cmc_equals_string(),
                        r#type: "text",
                        inputmode: "decimal",
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
                },
                FilterMode::Range => rsx! {
                    div { class: "flex-row gap-1 flex-center mb-1",
                        input { class: "input input-compact input-narrow",
                            id: "cmc-range-min",
                            placeholder: "min",
                            value: cmc_range_min_string(),
                            r#type: "text",
                            inputmode: "decimal",
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
                        span { class: "text-muted", "to" }
                        input { class: "input input-compact input-narrow",
                            id: "cmc-range-max",
                            placeholder: "max",
                            value: cmc_range_max_string(),
                            r#type: "text",
                            inputmode: "decimal",
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
                }
            }

            // Color identity filter
            div { class: "label-row",
                label { class: "label-xs", "color identity" }
                button {
                    class: "clear-btn",
                    onclick: move |_| {
                        let new_mode = color_identity_mode().toggle();
                        color_identity_mode.set(new_mode);
                        // When switching modes, move colors to the new mode field
                        let colors = if let Some(c) = filter_builder().color_identity_equals() {
                            c.to_vec()
                        } else if let Some(c) = filter_builder().color_identity_within() {
                            c.to_vec()
                        } else {
                            Vec::new()
                        };
                        let mut fb = filter_builder.write();
                        match new_mode {
                            FilterMode::Exact => {
                                fb.unset_color_identity_within();
                                if !colors.is_empty() {
                                    fb.set_color_identity_equals(colors.into());
                                }
                            }
                            FilterMode::Range => {
                                fb.unset_color_identity_equals();
                                if !colors.is_empty() {
                                    fb.set_color_identity_within(colors.into());
                                }
                            }
                        }
                    },
                    "{color_identity_mode().to_string().to_lowercase()}"
                }
                if color_is_active {
                    button {
                        class: "clear-btn",
                        onclick: move |_| {
                            let mut fb = filter_builder.write();
                            fb.unset_color_identity_equals();
                            fb.unset_color_identity_within();
                        },
                        "×"
                    }
                }
            }

            div { class: "flex flex-wrap gap-1 mb-1 flex-center",
                for color in Color::all() {
                    div {
                        class: if selected_colors.contains(&color) {
                            "chip selected"
                        } else {
                            "chip"
                        },
                        onclick: move |_| {
                            // Get current colors and toggle
                            let mut colors = if let Some(c) = filter_builder().color_identity_equals() {
                                c.to_vec()
                            } else if let Some(c) = filter_builder().color_identity_within() {
                                c.to_vec()
                            } else {
                                Vec::new()
                            };

                            if colors.contains(&color) {
                                colors.retain(|c| c != &color);
                            } else {
                                colors.push(color);
                            }

                            // Write to appropriate field based on mode
                            let mut fb = filter_builder.write();
                            match color_identity_mode() {
                                FilterMode::Exact => {
                                    if colors.is_empty() {
                                        fb.unset_color_identity_equals();
                                    } else {
                                        fb.set_color_identity_equals(colors.into());
                                    }
                                }
                                FilterMode::Range => {
                                    if colors.is_empty() {
                                        fb.unset_color_identity_within();
                                    } else {
                                        fb.set_color_identity_within(colors.into());
                                    }
                                }
                            }
                        },
                        { color.to_string().to_lowercase() }
                    }
                }
            }

            if let Some(error) = error() {
                div { class: "message-error", "{error}" }
            }
        }
    }
}
