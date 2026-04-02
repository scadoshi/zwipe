//! CMC (converted mana cost) filter component.

use dioxus::prelude::*;
use dioxus_primitives::toast::{use_toast, ToastOptions};
use std::time::Duration;
use zwipe::domain::card::models::search_card::card_filter::builder::CardFilterBuilder;

use super::super::filter_mode::FilterMode;

/// CMC filter sub-component.
#[component]
pub(crate) fn CmcFilter() -> Element {
    let mut filter_builder: Signal<CardFilterBuilder> = use_context();

    let toast = use_toast();

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
            return;
        }
        if let Ok(n) = cmc_equals_string().parse::<f64>() {
            filter_builder.write().set_cmc_equals(n);
            cmc_equals_string.set(n.to_string());
        } else {
            toast.error("invalid cmc".to_string(), ToastOptions::default().duration(Duration::from_millis(2000)));
        }
    };

    // Parse and write CMC range on blur
    let mut try_parse_cmc_range = move || {
        if cmc_range_min_string().is_empty() && cmc_range_max_string().is_empty() {
            filter_builder.write().unset_cmc_range();
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
        } else {
            toast.error("invalid cmc range".to_string(), ToastOptions::default().duration(Duration::from_millis(2000)));
        }
    };

    // Check if CMC filter is active (read directly from filter_builder)
    let cmc_is_active =
        filter_builder().cmc_equals().is_some() || filter_builder().cmc_range().is_some();

    rsx! {
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
                    },
                    "\u{00d7}"
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
                            cmc_range_max_string.set(event.value())
                        },
                        onblur: move |_| {
                            try_parse_cmc_range();
                        }
                    }
                }
            }
        }
    }
}
