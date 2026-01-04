use dioxus::prelude::*;
use zwipe::domain::card::models::search_card::card_filter::builder::CardFilterBuilder;

use super::filter_mode::FilterMode;

#[component]
pub fn Combat() -> Element {
    let mut filter_builder: Signal<CardFilterBuilder> = use_context();

    // Power filter mode and values (Option<i32> where None = no filter)
    let mut power_mode = use_signal(|| {
        if filter_builder().power_range().is_some() {
            FilterMode::Within
        } else {
            FilterMode::Exact
        }
    });
    let mut power_equals = use_signal(|| filter_builder().power_equals());
    let mut power_min = use_signal(|| filter_builder().power_range().map(|(min, _)| min));
    let mut power_max = use_signal(|| filter_builder().power_range().map(|(_, max)| max));

    // Toughness filter mode and values (Option<i32> where None = no filter)
    let mut toughness_mode = use_signal(|| {
        if filter_builder().toughness_range().is_some() {
            FilterMode::Within
        } else {
            FilterMode::Exact
        }
    });
    let mut toughness_equals = use_signal(|| filter_builder().toughness_equals());
    let mut toughness_min = use_signal(|| filter_builder().toughness_range().map(|(min, _)| min));
    let mut toughness_max = use_signal(|| filter_builder().toughness_range().map(|(_, max)| max));

    // Sync power to filter_builder (batch writes to avoid multiple notifications)
    use_effect(move || {
        let mode = power_mode();
        let mut fb = filter_builder.write();

        match mode {
            FilterMode::Exact => {
                fb.unset_power_range();
                if let Some(eq) = power_equals() {
                    fb.set_power_equals(eq);
                } else {
                    fb.unset_power_equals();
                }
            }
            FilterMode::Within => {
                fb.unset_power_equals();
                // If either min or max is set, use 0 as default for the other
                let min = power_min();
                let max = power_max();
                if min.is_some() || max.is_some() {
                    fb.set_power_range((min.unwrap_or(0), max.unwrap_or(0)));
                } else {
                    fb.unset_power_range();
                }
            }
        }
        // fb drops here - single notification
    });

    // Sync toughness to filter_builder (batch writes to avoid multiple notifications)
    use_effect(move || {
        let mode = toughness_mode();
        let mut fb = filter_builder.write();

        match mode {
            FilterMode::Exact => {
                fb.unset_toughness_range();
                if let Some(eq) = toughness_equals() {
                    fb.set_toughness_equals(eq);
                } else {
                    fb.unset_toughness_equals();
                }
            }
            FilterMode::Within => {
                fb.unset_toughness_equals();
                // If either min or max is set, use 0 as default for the other
                let min = toughness_min();
                let max = toughness_max();
                if min.is_some() || max.is_some() {
                    fb.set_toughness_range((min.unwrap_or(0), max.unwrap_or(0)));
                } else {
                    fb.unset_toughness_range();
                }
            }
        }
        // fb drops here - single notification
    });

    // Sync FROM filter_builder (handles clear_all)
    use_effect(move || {
        let fb = filter_builder();
        if fb.power_equals().is_none() && fb.power_range().is_none() {
            power_equals.set(None);
            power_min.set(None);
            power_max.set(None);
        }
        if fb.toughness_equals().is_none() && fb.toughness_range().is_none() {
            toughness_equals.set(None);
            toughness_min.set(None);
            toughness_max.set(None);
        }
    });

    // Check if power filter is active
    let power_is_active = power_equals().is_some()
        || (power_min().is_some() && power_max().is_some());

    // Check if toughness filter is active
    let toughness_is_active = toughness_equals().is_some()
        || (toughness_min().is_some() && toughness_max().is_some());

    rsx! {
        div { class: "flex-col gap-half",
            // Power filter
            div { class: "label-row",
                label { class: "label-xs", "power" }
                button {
                    class: "clear-btn",
                    onclick: move |_| {
                        power_mode.set(power_mode().toggle());
                    },
                    "{power_mode()}"
                }
                if power_is_active {
                    button {
                        class: "clear-btn",
                        onclick: move |_| {
                            power_equals.set(None);
                            power_min.set(None);
                            power_max.set(None);
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
                                let val = power_equals().unwrap_or(0);
                                power_equals.set(Some(val.saturating_sub(1)));
                            },
                            "-"
                        }
                        span { class: "stepper-value",
                            if let Some(v) = power_equals() { "{v}" } else { "-" }
                        }
                        button {
                            class: "stepper-btn",
                            onclick: move |_| {
                                let val = power_equals().unwrap_or(0);
                                power_equals.set(Some(val.saturating_add(1)));
                            },
                            "+"
                        }
                    }
                },
                FilterMode::Within => rsx! {
                    div { class: "flex-row gap-2 flex-center",
                        div { class: "stepper",
                            button {
                                class: "stepper-btn",
                                onclick: move |_| {
                                    let val = power_min().unwrap_or(0);
                                    power_min.set(Some(val.saturating_sub(1)));
                                },
                                "-"
                            }
                            span { class: "stepper-value",
                                if let Some(v) = power_min() { "{v}" } else { "-" }
                            }
                            button {
                                class: "stepper-btn",
                                onclick: move |_| {
                                    let val = power_min().unwrap_or(0);
                                    power_min.set(Some(val.saturating_add(1)));
                                },
                                "+"
                            }
                        }
                        span { class: "text-muted", "to" }
                        div { class: "stepper",
                            button {
                                class: "stepper-btn",
                                onclick: move |_| {
                                    let val = power_max().unwrap_or(0);
                                    power_max.set(Some(val.saturating_sub(1)));
                                },
                                "-"
                            }
                            span { class: "stepper-value",
                                if let Some(v) = power_max() { "{v}" } else { "-" }
                            }
                            button {
                                class: "stepper-btn",
                                onclick: move |_| {
                                    let val = power_max().unwrap_or(0);
                                    power_max.set(Some(val.saturating_add(1)));
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
                        toughness_mode.set(toughness_mode().toggle());
                    },
                    "{toughness_mode()}"
                }
                if toughness_is_active {
                    button {
                        class: "clear-btn",
                        onclick: move |_| {
                            toughness_equals.set(None);
                            toughness_min.set(None);
                            toughness_max.set(None);
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
                                let val = toughness_equals().unwrap_or(0);
                                toughness_equals.set(Some(val.saturating_sub(1)));
                            },
                            "-"
                        }
                        span { class: "stepper-value",
                            if let Some(v) = toughness_equals() { "{v}" } else { "-" }
                        }
                        button {
                            class: "stepper-btn",
                            onclick: move |_| {
                                let val = toughness_equals().unwrap_or(0);
                                toughness_equals.set(Some(val.saturating_add(1)));
                            },
                            "+"
                        }
                    }
                },
                FilterMode::Within => rsx! {
                    div { class: "flex-row gap-2 flex-center",
                        div { class: "stepper",
                            button {
                                class: "stepper-btn",
                                onclick: move |_| {
                                    let val = toughness_min().unwrap_or(0);
                                    toughness_min.set(Some(val.saturating_sub(1)));
                                },
                                "-"
                            }
                            span { class: "stepper-value",
                                if let Some(v) = toughness_min() { "{v}" } else { "-" }
                            }
                            button {
                                class: "stepper-btn",
                                onclick: move |_| {
                                    let val = toughness_min().unwrap_or(0);
                                    toughness_min.set(Some(val.saturating_add(1)));
                                },
                                "+"
                            }
                        }
                        span { class: "text-muted", "to" }
                        div { class: "stepper",
                            button {
                                class: "stepper-btn",
                                onclick: move |_| {
                                    let val = toughness_max().unwrap_or(0);
                                    toughness_max.set(Some(val.saturating_sub(1)));
                                },
                                "-"
                            }
                            span { class: "stepper-value",
                                if let Some(v) = toughness_max() { "{v}" } else { "-" }
                            }
                            button {
                                class: "stepper-btn",
                                onclick: move |_| {
                                    let val = toughness_max().unwrap_or(0);
                                    toughness_max.set(Some(val.saturating_add(1)));
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
