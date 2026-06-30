//! Toughness filter sub-component.

use dioxus::prelude::*;
use zwipe_core::domain::card::search_card::card_filter::builder::CardFilterBuilder;

use super::super::filter_mode::FilterMode;

/// Filter component for card toughness values.
#[component]
pub(crate) fn ToughnessFilter() -> Element {
    let mut filter_builder: Signal<CardFilterBuilder> = use_context();

    let mut toughness_mode = use_signal(|| {
        if filter_builder().toughness_range().is_some() {
            FilterMode::Range
        } else {
            FilterMode::Exact
        }
    });

    let toughness_is_active = filter_builder().toughness_equals().is_some()
        || filter_builder().toughness_range().is_some();

    rsx! {
        div { class: "label-row mt-2",
            label { class: "label-xs", "Toughness" }
            button {
                class: "chip-xs",
                onclick: move |_| {
                    let new_mode = toughness_mode().toggle();
                    toughness_mode.set(new_mode);
                    let mut fb = filter_builder.write();
                    match new_mode {
                        FilterMode::Exact => {
                            fb.unset_toughness_range();
                        }
                        FilterMode::Range => {
                            fb.unset_toughness_equals();
                        }
                    }
                },
                "{toughness_mode()}"
            }
            if toughness_is_active {
                button {
                    class: "clear-btn",
                    onclick: move |_| {
                        let mut fb = filter_builder.write();
                        fb.unset_toughness_equals();
                        fb.unset_toughness_range();
                    },
                    "\u{00d7}"
                }
            }
        }

        match toughness_mode() {
            FilterMode::Exact => rsx! {
                div { class: "stepper",
                    button {
                        class: "stepper-btn",
                        onclick: move |_| {
                            let current = filter_builder().toughness_equals().unwrap_or(0);
                            filter_builder.write().set_toughness_equals(current.saturating_sub(1));
                        },
                        "-"
                    }
                    span { class: "stepper-value",
                        if let Some(v) = filter_builder().toughness_equals() { "{v}" } else { "-" }
                    }
                    button {
                        class: "stepper-btn",
                        onclick: move |_| {
                            let current = filter_builder().toughness_equals().unwrap_or(0);
                            filter_builder.write().set_toughness_equals(current.saturating_add(1));
                        },
                        "+"
                    }
                }
            },
            FilterMode::Range => rsx! {
                div { class: "flex-row gap-2 flex-center",
                    div { class: "stepper",
                        button {
                            class: "stepper-btn",
                            onclick: move |_| {
                                let (min, max) = filter_builder().toughness_range().unwrap_or((0, 0));
                                filter_builder.write().set_toughness_range((min.saturating_sub(1), max));
                            },
                            "-"
                        }
                        span { class: "stepper-value",
                            if let Some((min, _)) = filter_builder().toughness_range() { "{min}" } else { "-" }
                        }
                        button {
                            class: "stepper-btn",
                            onclick: move |_| {
                                let (min, max) = filter_builder().toughness_range().unwrap_or((0, 0));
                                filter_builder.write().set_toughness_range((min.saturating_add(1), max));
                            },
                            "+"
                        }
                    }
                    span { class: "text-muted", "to" }
                    div { class: "stepper",
                        button {
                            class: "stepper-btn",
                            onclick: move |_| {
                                let (min, max) = filter_builder().toughness_range().unwrap_or((0, 0));
                                filter_builder.write().set_toughness_range((min, max.saturating_sub(1)));
                            },
                            "-"
                        }
                        span { class: "stepper-value",
                            if let Some((_, max)) = filter_builder().toughness_range() { "{max}" } else { "-" }
                        }
                        button {
                            class: "stepper-btn",
                            onclick: move |_| {
                                let (min, max) = filter_builder().toughness_range().unwrap_or((0, 0));
                                filter_builder.write().set_toughness_range((min, max.saturating_add(1)));
                            },
                            "+"
                        }
                    }
                }
            }
        }
    }
}
