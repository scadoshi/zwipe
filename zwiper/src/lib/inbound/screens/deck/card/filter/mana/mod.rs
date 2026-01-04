use dioxus::prelude::*;
use zwipe::domain::card::models::scryfall_data::colors::Color;
use zwipe::domain::card::models::search_card::card_filter::builder::CardFilterBuilder;

use super::filter_mode::FilterMode;

#[component]
pub fn Mana() -> Element {
    let mut filter_builder: Signal<CardFilterBuilder> = use_context();

    let mut error = use_signal(|| None::<String>);

    // CMC mode and values
    let mut cmc_mode = use_signal(|| {
        if filter_builder().cmc_range().is_some() {
            FilterMode::Within
        } else {
            FilterMode::Exact
        }
    });

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

    // Color identity mode and values
    let mut selected_colors = use_signal(|| {
        if let Some(colors) = filter_builder().color_identity_equals() {
            colors.to_vec()
        } else if let Some(colors) = filter_builder().color_identity_within() {
            colors.to_vec()
        } else {
            Vec::new()
        }
    });
    let mut color_identity_mode = use_signal(|| {
        if filter_builder().color_identity_equals().is_some() {
            FilterMode::Exact
        } else if filter_builder().color_identity_within().is_some() {
            FilterMode::Within
        } else {
            FilterMode::default()
        }
    });

    // Sync local colors TO filter_builder (only if changed)
    use_effect(move || {
        let colors = selected_colors();
        let mode = color_identity_mode();
        let current_equals = filter_builder().color_identity_equals().map(|v| v.to_vec());
        let current_within = filter_builder().color_identity_within().map(|v| v.to_vec());

        if colors.is_empty() {
            if current_equals.is_some() {
                filter_builder.write().unset_color_identity_equals();
            }
            if current_within.is_some() {
                filter_builder.write().unset_color_identity_within();
            }
        } else {
            match mode {
                FilterMode::Exact => {
                    if current_within.is_some() {
                        filter_builder.write().unset_color_identity_within();
                    }
                    if current_equals.as_ref() != Some(&colors) {
                        filter_builder.write().set_color_identity_equals(colors.into());
                    }
                }
                FilterMode::Within => {
                    if current_equals.is_some() {
                        filter_builder.write().unset_color_identity_equals();
                    }
                    if current_within.as_ref() != Some(&colors) {
                        filter_builder.write().set_color_identity_within(colors.into());
                    }
                }
            }
        }
    });

    // Track previous filter_builder CMC state to detect external clears
    let mut prev_cmc_equals = use_signal(|| filter_builder().cmc_equals());
    let mut prev_cmc_range = use_signal(|| filter_builder().cmc_range());

    // Sync FROM filter_builder (handles clear_all)
    // Only clear local values if filter_builder WAS set and is NOW unset (external clear)
    use_effect(move || {
        let fb = filter_builder();
        let current_cmc_equals = fb.cmc_equals();
        let current_cmc_range = fb.cmc_range();

        // CMC equals: only clear if it WAS set and is now None
        if prev_cmc_equals().is_some() && current_cmc_equals.is_none() {
            cmc_equals_string.set(String::new());
        }
        prev_cmc_equals.set(current_cmc_equals);

        // CMC range: only clear if it WAS set and is now None
        if prev_cmc_range().is_some() && current_cmc_range.is_none() {
            cmc_range_min_string.set(String::new());
            cmc_range_max_string.set(String::new());
        }
        prev_cmc_range.set(current_cmc_range);

        // Color identity: clear when both are None (simpler since it syncs immediately)
        if fb.color_identity_equals().is_none() && fb.color_identity_within().is_none() {
            selected_colors.set(Vec::new());
            color_identity_mode.set(FilterMode::default());
        }
    });

    // Check if CMC filter is active
    let cmc_is_active =
        filter_builder().cmc_equals().is_some() || filter_builder().cmc_range().is_some();

    rsx! {
        div { class: "flex-col gap-half",
            // CMC filter
            div { class: "label-row",
                label { class: "label-xs", "cmc" }
                button {
                    class: "clear-btn",
                    onclick: move |_| {
                        cmc_mode.set(cmc_mode().toggle());
                    },
                    "{cmc_mode()}"
                }
                if cmc_is_active {
                    button {
                        class: "clear-btn",
                        onclick: move |_| {
                            filter_builder.write().unset_cmc_equals();
                            filter_builder.write().unset_cmc_range();
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
                FilterMode::Within => rsx! {
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
                        color_identity_mode.set(color_identity_mode().toggle());
                    },
                    "{color_identity_mode()}"
                }
                if filter_builder().color_identity_equals().is_some() || filter_builder().color_identity_within().is_some() {
                    button {
                        class: "clear-btn",
                        onclick: move |_| {
                            filter_builder.write().unset_color_identity_equals();
                            filter_builder.write().unset_color_identity_within();
                        },
                        "×"
                    }
                }
            }

            div { class: "flex flex-wrap gap-1 mb-1 flex-center",
                for color in Color::all() {
                    div {
                        class: if selected_colors().contains(&color) {
                            "chip selected"
                        } else {
                            "chip"
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

            if let Some(error) = error() {
                div { class: "message-error", "{error}" }
            }
        }
    }
}
