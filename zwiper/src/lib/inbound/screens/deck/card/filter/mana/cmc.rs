//! CMC (converted mana cost) filter component.

use components::{TOAST_QUICK, ToastOptions, use_toast};
use dioxus::prelude::*;
use zwipe_core::domain::card::search_card::card_filter::builder::CardQueryBuilder;

use super::super::filter_mode::FilterMode;
use super::super::typed_number::TypedNumber;

/// CMC filter sub-component.
#[component]
pub(crate) fn CmcFilter() -> Element {
    let mut filter_builder: Signal<CardQueryBuilder> = use_context();

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
            .map_or_default(|v| v.to_string())
    });
    let mut cmc_range_min_string = use_signal(|| {
        filter_builder()
            .cmc_range()
            .map_or_default(|(min, _)| min.to_string())
    });
    let mut cmc_range_max_string = use_signal(|| {
        filter_builder()
            .cmc_range()
            .map_or_default(|(_, max)| max.to_string())
    });

    // Parse and write CMC equals on blur, tidying the text or saying it is invalid
    let mut try_parse_cmc_equals = move || {
        if cmc_equals_string().is_empty() {
            filter_builder.write().unset_cmc_equals();
            return;
        }
        if let Ok(n) = cmc_equals_string().parse::<f64>() {
            filter_builder.write().set_cmc_equals(n);
            cmc_equals_string.set(n.to_string());
        } else {
            toast.error(
                "Invalid mana value".to_string(),
                ToastOptions::default().duration(TOAST_QUICK),
            );
        }
    };

    // Parse and write CMC range on blur, tidying the text or saying it is invalid
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
            toast.error(
                "Invalid mana value range".to_string(),
                ToastOptions::default().duration(TOAST_QUICK),
            );
        }
    };

    // Writes the exact value as it is typed; half-typed text waits for the
    // blur, which says it is invalid.
    let mut write_typed_equals = move |text: &str| match TypedNumber::read(text) {
        TypedNumber::Number(n) => {
            filter_builder.write().set_cmc_equals(n);
        }
        TypedNumber::Empty => {
            filter_builder.write().unset_cmc_equals();
        }
        TypedNumber::Partial => {}
    };

    // Writes the range as it is typed once both ends read as numbers, and
    // drops it once both are empty.
    let mut write_typed_range = move || match (
        TypedNumber::read(&cmc_range_min_string.peek()),
        TypedNumber::read(&cmc_range_max_string.peek()),
    ) {
        (TypedNumber::Number(min), TypedNumber::Number(max)) => {
            filter_builder.write().set_cmc_range((min, max));
        }
        (TypedNumber::Empty, TypedNumber::Empty) => {
            filter_builder.write().unset_cmc_range();
        }
        _ => {}
    };

    // Check if CMC filter is active (read directly from filter_builder)
    let cmc_is_active =
        filter_builder().cmc_equals().is_some() || filter_builder().cmc_range().is_some();

    rsx! {
        // CMC filter
        div { class: "label-row mt-2",
            label { class: "label-xs", "Mana value" }
            button {
                class: "chip-xs",
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
                "{cmc_mode()}"
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
                input { class: "input input-compact input-narrow input-number-single",
                    id: "cmc-equals",
                    placeholder: "Mana value",
                    value: cmc_equals_string(),
                    r#type: "text",
                    inputmode: "decimal",
                    autocapitalize: "none",
                    autocorrect: "off",
                    spellcheck: "false",
                    oninput: move |event| {
                        let text = event.value();
                        write_typed_equals(&text);
                        cmc_equals_string.set(text);
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
                        placeholder: "Min",
                        value: cmc_range_min_string(),
                        r#type: "text",
                        inputmode: "decimal",
                        autocapitalize: "none",
                        autocorrect: "off",
                        spellcheck: "false",
                        oninput: move |event| {
                            cmc_range_min_string.set(event.value());
                            write_typed_range();
                        },
                        onblur: move |_| {
                            try_parse_cmc_range();
                        }
                    }
                    span { class: "text-muted", "to" }
                    input { class: "input input-compact input-narrow",
                        id: "cmc-range-max",
                        placeholder: "Max",
                        value: cmc_range_max_string(),
                        r#type: "text",
                        inputmode: "decimal",
                        autocapitalize: "none",
                        autocorrect: "off",
                        spellcheck: "false",
                        oninput: move |event| {
                            cmc_range_max_string.set(event.value());
                            write_typed_range();
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
