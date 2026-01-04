use dioxus::prelude::*;
use zwipe::domain::card::models::search_card::card_filter::builder::CardFilterBuilder;

use super::filter_mode::FilterMode;

#[component]
pub fn Combat() -> Element {
    let mut filter_builder: Signal<CardFilterBuilder> = use_context();

    // Mode signals - needed for Exact/Range toggle UI state
    let mut power_mode = use_signal(|| {
        if filter_builder().power_range().is_some() {
            FilterMode::Range
        } else {
            FilterMode::Exact
        }
    });
    let mut toughness_mode = use_signal(|| {
        if filter_builder().toughness_range().is_some() {
            FilterMode::Range
        } else {
            FilterMode::Exact
        }
    });

    // Check if power filter is active (read directly from filter_builder)
    let power_is_active =
        filter_builder().power_equals().is_some() || filter_builder().power_range().is_some();

    // Check if toughness filter is active (read directly from filter_builder)
    let toughness_is_active = filter_builder().toughness_equals().is_some()
        || filter_builder().toughness_range().is_some();

    rsx! {
        div { class: "flex-col gap-half",
            // Power filter
            div { class: "label-row",
                label { class: "label-xs", "power" }
                button {
                    class: "clear-btn",
                    onclick: move |_| {
                        let new_mode = power_mode().toggle();
                        power_mode.set(new_mode);
                        // Clear the other mode's value when switching
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
                    "{power_mode().to_string().to_lowercase()}"
                }
                if power_is_active {
                    button {
                        class: "clear-btn",
                        onclick: move |_| {
                            let mut fb = filter_builder.write();
                            fb.unset_power_equals();
                            fb.unset_power_range();
                        },
                        "×"
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

            // Toughness filter
            div { class: "label-row mt-2",
                label { class: "label-xs", "toughness" }
                button {
                    class: "clear-btn",
                    onclick: move |_| {
                        let new_mode = toughness_mode().toggle();
                        toughness_mode.set(new_mode);
                        // Clear the other mode's value when switching
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
                    "{toughness_mode().to_string().to_lowercase()}"
                }
                if toughness_is_active {
                    button {
                        class: "clear-btn",
                        onclick: move |_| {
                            let mut fb = filter_builder.write();
                            fb.unset_toughness_equals();
                            fb.unset_toughness_range();
                        },
                        "×"
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
}
