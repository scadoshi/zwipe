//! Color identity filter component.

use dioxus::prelude::*;
use zwipe::domain::card::models::scryfall_data::colors::Color;
use zwipe::domain::card::models::search_card::card_filter::builder::CardFilterBuilder;

use super::super::filter_mode::FilterMode;

/// Color identity filter sub-component.
#[component]
pub(crate) fn ColorIdentityFilter() -> Element {
    let mut filter_builder: Signal<CardFilterBuilder> = use_context();

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
        // Color identity filter
        div { class: "label-row mt-2",
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
                    "\u{00d7}"
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
    }
}
