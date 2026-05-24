//! Power filter sub-component.

use dioxus::prelude::*;
use zwipe_core::domain::card::search_card::card_filter::builder::CardFilterBuilder;

use super::super::filter_mode::FilterMode;

/// Filter component for card power values.
#[component]
pub(crate) fn PowerFilter() -> Element {
    let mut filter_builder: Signal<CardFilterBuilder> = use_context();

    let mut power_mode = use_signal(|| {
        if filter_builder().power_range().is_some() {
            FilterMode::Range
        } else {
            FilterMode::Exact
        }
    });

    let power_is_active =
        filter_builder().power_equals().is_some() || filter_builder().power_range().is_some();

    rsx! {
        div { class: "label-row mt-2",
            label { class: "label-xs", "Power" }
            button {
                class: "clear-btn",
                onclick: move |_| {
                    let new_mode = power_mode().toggle();
                    power_mode.set(new_mode);
                    let mut fb = filter_builder.write();
                    match new_mode {
                        FilterMode::Exact => {
                            fb.unset_power_range();
                        }
                        FilterMode::Range => {
                            fb.unset_power_equals();
                        }
                    }
                },
                "{power_mode()}"
            }
            if power_is_active {
                button {
                    class: "clear-btn",
                    onclick: move |_| {
                        let mut fb = filter_builder.write();
                        fb.unset_power_equals();
                        fb.unset_power_range();
                    },
                    "\u{00d7}"
                }
            }
        }

        match power_mode() {
            FilterMode::Exact => rsx! {
                div { class: "stepper",
                    button {
                        class: "stepper-btn",
                        onclick: move |_| {
                            let current = filter_builder().power_equals().unwrap_or(0);
                            filter_builder.write().set_power_equals(current.saturating_sub(1));
                        },
                        "-"
                    }
                    span { class: "stepper-value",
                        if let Some(v) = filter_builder().power_equals() { "{v}" } else { "-" }
                    }
                    button {
                        class: "stepper-btn",
                        onclick: move |_| {
                            let current = filter_builder().power_equals().unwrap_or(0);
                            filter_builder.write().set_power_equals(current.saturating_add(1));
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
                                let (min, max) = filter_builder().power_range().unwrap_or((0, 0));
                                filter_builder.write().set_power_range((min.saturating_sub(1), max));
                            },
                            "-"
                        }
                        span { class: "stepper-value",
                            if let Some((min, _)) = filter_builder().power_range() { "{min}" } else { "-" }
                        }
                        button {
                            class: "stepper-btn",
                            onclick: move |_| {
                                let (min, max) = filter_builder().power_range().unwrap_or((0, 0));
                                filter_builder.write().set_power_range((min.saturating_add(1), max));
                            },
                            "+"
                        }
                    }
                    span { class: "text-muted", "to" }
                    div { class: "stepper",
                        button {
                            class: "stepper-btn",
                            onclick: move |_| {
                                let (min, max) = filter_builder().power_range().unwrap_or((0, 0));
                                filter_builder.write().set_power_range((min, max.saturating_sub(1)));
                            },
                            "-"
                        }
                        span { class: "stepper-value",
                            if let Some((_, max)) = filter_builder().power_range() { "{max}" } else { "-" }
                        }
                        button {
                            class: "stepper-btn",
                            onclick: move |_| {
                                let (min, max) = filter_builder().power_range().unwrap_or((0, 0));
                                filter_builder.write().set_power_range((min, max.saturating_add(1)));
                            },
                            "+"
                        }
                    }
                }
            }
        }
    }
}
