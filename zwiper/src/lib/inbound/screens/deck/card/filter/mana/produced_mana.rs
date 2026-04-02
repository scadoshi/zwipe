//! Produced mana filter component.

use dioxus::prelude::*;
use zwipe::domain::card::models::search_card::card_filter::builder::CardFilterBuilder;

use super::super::match_mode::MatchMode;

/// Mana symbols for the produced mana filter (WUBRG + colorless).
const MANA_SYMBOLS: &[(&str, &str)] = &[
    ("W", "white"),
    ("U", "blue"),
    ("B", "black"),
    ("R", "red"),
    ("G", "green"),
    ("C", "colorless"),
];

/// Read selected produced mana colors from the filter builder based on current mode.
fn read_produced_mana(fb: &CardFilterBuilder, mode: MatchMode) -> Vec<String> {
    match mode {
        MatchMode::Any => fb
            .produced_mana_contains_any()
            .map(|v| v.to_vec())
            .unwrap_or_default(),
        MatchMode::All => fb
            .produced_mana_contains_all()
            .map(|v| v.to_vec())
            .unwrap_or_default(),
    }
}

/// Write produced mana colors to the filter builder based on current mode.
fn write_produced_mana(fb: &mut CardFilterBuilder, mode: MatchMode, values: Vec<String>) {
    fb.unset_produced_mana_contains_any();
    fb.unset_produced_mana_contains_all();
    if !values.is_empty() {
        match mode {
            MatchMode::Any => {
                fb.set_produced_mana_contains_any(values);
            }
            MatchMode::All => {
                fb.set_produced_mana_contains_all(values);
            }
        }
    }
}

/// Produced mana filter sub-component.
#[component]
pub(crate) fn ProducedManaFilter() -> Element {
    let mut filter_builder: Signal<CardFilterBuilder> = use_context();

    // Produced mana mode signal (any vs all)
    let mut produced_mana_mode = use_signal(|| {
        if filter_builder().produced_mana_contains_all().is_some() {
            MatchMode::All
        } else {
            MatchMode::Any
        }
    });

    // Get current selected produced mana colors
    let selected_produced_mana = read_produced_mana(&filter_builder(), produced_mana_mode());

    rsx! {
        // Produced mana filter
        div { class: "label-row",
            label { class: "label-xs", "produces" }
            button {
                class: "clear-btn",
                onclick: move |_| {
                    let new_mode = produced_mana_mode().toggle();
                    let current = read_produced_mana(&filter_builder(), produced_mana_mode());
                    write_produced_mana(&mut filter_builder.write(), new_mode, current);
                    produced_mana_mode.set(new_mode);
                },
                "{produced_mana_mode().label()}"
            }
            if !selected_produced_mana.is_empty() {
                button {
                    class: "clear-btn",
                    onclick: move |_| {
                        write_produced_mana(&mut filter_builder.write(), produced_mana_mode(), vec![]);
                    },
                    "\u{00d7}"
                }
            }
        }

        div { class: "flex flex-wrap gap-1 mb-1 flex-center",
            for &(code, label) in MANA_SYMBOLS.iter() {
                div {
                    class: if selected_produced_mana.contains(&code.to_string()) {
                        "chip selected"
                    } else {
                        "chip"
                    },
                    onclick: move |_| {
                        let mode = produced_mana_mode();
                        let mut colors = read_produced_mana(&filter_builder(), mode);
                        let code_str = code.to_string();
                        if colors.contains(&code_str) {
                            colors.retain(|c| c != &code_str);
                        } else {
                            colors.push(code_str);
                        }
                        write_produced_mana(&mut filter_builder.write(), mode, colors);
                    },
                    "{label}"
                }
            }
        }
    }
}
