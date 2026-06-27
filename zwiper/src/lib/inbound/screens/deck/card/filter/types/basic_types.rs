//! Basic card type filter sub-component (creature, instant, etc.).

use super::super::match_mode::MatchMode;
use dioxus::prelude::*;
use zwipe_core::domain::card::search_card::card_filter::builder::CardFilterBuilder;
use zwipe_core::domain::card::search_card::card_type::{CardType, WithCardTypes};

fn read_card_types(fb: &CardFilterBuilder, mode: MatchMode) -> Vec<CardType> {
    match mode {
        MatchMode::Any => fb
            .card_type_contains_any()
            .map(|v| v.to_vec())
            .unwrap_or_default(),
        MatchMode::All => fb
            .card_type_contains_all()
            .map(|v| v.to_vec())
            .unwrap_or_default(),
    }
}

fn write_card_types(fb: &mut CardFilterBuilder, mode: MatchMode, values: Vec<CardType>) {
    fb.unset_card_type_contains_any();
    fb.unset_card_type_contains_all();
    if !values.is_empty() {
        match mode {
            MatchMode::Any => {
                fb.set_card_type_contains_any(values);
            }
            MatchMode::All => {
                fb.set_card_type_contains_all(values);
            }
        }
    }
}

fn read_excluded(fb: &CardFilterBuilder) -> Vec<CardType> {
    fb.card_type_excludes_any()
        .map(|v| v.to_vec())
        .unwrap_or_default()
}

fn write_excluded(fb: &mut CardFilterBuilder, values: Vec<CardType>) {
    if values.is_empty() {
        fb.unset_card_type_excludes_any();
    } else {
        fb.set_card_type_excludes_any(values);
    }
}

/// Basic card type filter with separate include and exclude chip grids.
#[component]
pub(crate) fn BasicTypes() -> Element {
    let mut filter_builder: Signal<CardFilterBuilder> = use_context();

    let mut card_type_mode = use_signal(|| {
        if filter_builder().card_type_contains_all().is_some() {
            MatchMode::All
        } else {
            MatchMode::Any
        }
    });

    let selected = read_card_types(&filter_builder(), card_type_mode());
    let excluded = read_excluded(&filter_builder());

    rsx! {
        // ── basic types (include) ─────────────────────────────────
        div { class: "label-row mt-2",
            label { class: "label-xs", r#for: "card-type", "Basic types include" }
            if !selected.is_empty() {
                button {
                    class: "clear-btn",
                    onclick: move |_| {
                        let new_mode = card_type_mode().toggle();
                        let current = read_card_types(&filter_builder(), card_type_mode());
                        write_card_types(&mut filter_builder.write(), new_mode, current);
                        card_type_mode.set(new_mode);
                    },
                    "{card_type_mode().label()}"
                }
            }
            if !selected.is_empty() {
                button {
                    class: "clear-btn",
                    onclick: move |_| {
                        write_card_types(&mut filter_builder.write(), card_type_mode(), vec![]);
                    },
                    "×"
                }
            }
        }
        div { class: "flex flex-wrap gap-1 mb-1 flex-center",
            for card_type in Vec::with_all_card_types() {
                div {
                    class: if selected.contains(&card_type) { "chip selected" } else { "chip" },
                    onclick: move |_| {
                        let mode = card_type_mode();
                        let mut new = read_card_types(&filter_builder(), mode);
                        if new.contains(&card_type) {
                            new.retain(|x| x != &card_type);
                        } else {
                            new.push(card_type);
                        }
                        write_card_types(&mut filter_builder.write(), mode, new);
                    },
                    "{card_type}"
                }
            }
        }

        // ── basic types excludes ──────────────────────────────────
        div { class: "label-row mt-2",
            label { class: "label-xs", "Basic types exclude" }
            if !excluded.is_empty() {
                button {
                    class: "clear-btn",
                    onclick: move |_| {
                        write_excluded(&mut filter_builder.write(), vec![]);
                    },
                    "×"
                }
            }
        }
        div { class: "flex flex-wrap gap-1 mb-1 flex-center",
            for card_type in Vec::with_all_card_types() {
                div {
                    class: if excluded.contains(&card_type) { "chip selected" } else { "chip" },
                    onclick: move |_| {
                        let mut exc = read_excluded(&filter_builder());
                        if exc.contains(&card_type) {
                            exc.retain(|x| x != &card_type);
                        } else {
                            exc.push(card_type);
                        }
                        write_excluded(&mut filter_builder.write(), exc);
                    },
                    "{card_type}"
                }
            }
        }
    }
}
