//! Produced mana filter component.

use dioxus::prelude::*;
use zwipe_core::domain::card::search_card::card_filter::builder::CardQueryBuilder;
use zwipe_ui::filter::chips;
use zwipe_ui::filter::sections::{ChipSet, Modes};

use super::super::match_mode::MatchMode;

fn read_produced_mana(fb: &CardQueryBuilder, mode: MatchMode) -> Vec<String> {
    match mode {
        MatchMode::Any => fb
            .produced_mana_contains_any()
            .map_or_default(<[_]>::to_vec),
        MatchMode::All => fb
            .produced_mana_contains_all()
            .map_or_default(<[_]>::to_vec),
    }
}

fn write_produced_mana(fb: &mut CardQueryBuilder, mode: MatchMode, values: Vec<String>) {
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

fn read_excluded(fb: &CardQueryBuilder) -> Vec<String> {
    fb.produced_mana_excludes().map_or_default(<[_]>::to_vec)
}

fn write_excluded(fb: &mut CardQueryBuilder, values: Vec<String>) {
    if values.is_empty() {
        fb.unset_produced_mana_excludes();
    } else {
        fb.set_produced_mana_excludes(values);
    }
}

/// Produced mana filter with separate include and exclude chip grids.
#[component]
pub(crate) fn ProducedManaFilter() -> Element {
    let mut filter_builder: Signal<CardQueryBuilder> = use_context();

    let mut produced_mana_mode = use_signal(|| {
        if filter_builder().produced_mana_contains_all().is_some() {
            MatchMode::All
        } else {
            MatchMode::Any
        }
    });

    let selected = read_produced_mana(&filter_builder(), produced_mana_mode());
    let excluded = read_excluded(&filter_builder());

    rsx! {
        // == produces (include) ==
        div { class: "label-row mt-2",
            label { class: "label-xs", "Produces" }
            button {
                class: "chip-xs",
                onclick: move |_| {
                    let new_mode = produced_mana_mode().toggle();
                    let current = read_produced_mana(&filter_builder(), produced_mana_mode());
                    write_produced_mana(&mut filter_builder.write(), new_mode, current);
                    produced_mana_mode.set(new_mode);
                },
                "{produced_mana_mode().label()}"
            }
            if !selected.is_empty() {
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
            for (code, label) in chips::PRODUCED {
                div {
                    class: if selected.contains(&code.to_string()) { "chip selected" } else { "chip" },
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
                    title: "{label}",
                    i { class: "ms ms-{code.to_lowercase()} ms-cost" }
                }
            }
        }

        // == does not produce (exclude) ==
        div { class: "label-row mt-2",
            label { class: "label-xs", "Doesn't produce" }
            if !excluded.is_empty() {
                button {
                    class: "clear-btn",
                    onclick: move |_| {
                        write_excluded(&mut filter_builder.write(), vec![]);
                    },
                    "\u{00d7}"
                }
            }
        }

        div { class: "flex flex-wrap gap-1 mb-1 flex-center",
            for (code, label) in chips::PRODUCED {
                div {
                    class: if chips::is_selected(
                        ChipSet::ProducedExcludes,
                        &filter_builder(),
                        code,
                    ) {
                        "chip selected"
                    } else {
                        "chip"
                    },
                    onclick: move |_| {
                        chips::tap(
                            ChipSet::ProducedExcludes,
                            &mut filter_builder.write(),
                            &Modes::default(),
                            code,
                        );
                    },
                    title: "{label}",
                    i { class: "ms ms-{code.to_lowercase()} ms-cost" }
                }
            }
        }
    }
}
