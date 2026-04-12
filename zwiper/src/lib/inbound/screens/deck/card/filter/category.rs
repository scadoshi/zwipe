//! Mechanical category filter component.

use dioxus::prelude::*;
use zwipe_core::domain::card::{
    mechanical_category::MechanicalCategory,
    search_card::card_filter::builder::CardFilterBuilder,
};

use super::match_mode::MatchMode;

/// Filter component for mechanical categories with separate include and exclude grids.
#[component]
pub fn Category() -> Element {
    let mut filter_builder: Signal<CardFilterBuilder> = use_context();

    let mode = use_memo(move || {
        let fb = filter_builder();
        if fb.mechanical_categories_contains_all().is_some() {
            MatchMode::All
        } else {
            MatchMode::Any
        }
    });

    let read_selected = move || -> Vec<String> {
        let fb = filter_builder();
        match mode() {
            MatchMode::Any => fb.mechanical_categories_contains_any().map(|v| v.to_vec()).unwrap_or_default(),
            MatchMode::All => fb.mechanical_categories_contains_all().map(|v| v.to_vec()).unwrap_or_default(),
        }
    };

    let read_excluded = move || -> Vec<String> {
        filter_builder().mechanical_categories_excludes().map(|v| v.to_vec()).unwrap_or_default()
    };

    let mut write_categories = move |cats: Vec<String>, m: MatchMode| {
        let fb = &mut *filter_builder.write();
        fb.unset_mechanical_categories_contains_any();
        fb.unset_mechanical_categories_contains_all();
        if !cats.is_empty() {
            match m {
                MatchMode::Any => { fb.set_mechanical_categories_contains_any(cats); }
                MatchMode::All => { fb.set_mechanical_categories_contains_all(cats); }
            }
        }
    };

    let mut write_excluded_cats = move |cats: Vec<String>| {
        let fb = &mut *filter_builder.write();
        if cats.is_empty() {
            fb.unset_mechanical_categories_excludes();
        } else {
            fb.set_mechanical_categories_excludes(cats);
        }
    };

    let selected = read_selected();
    let excluded = read_excluded();

    rsx! {
        div { class: "flex-col gap-half",
            // ── category includes ─────────────────────────────────
            div { class: "label-row mt-2",
                label { class: "label-xs", "category includes" }
                if !selected.is_empty() {
                    button {
                        class: "chip-xs",
                        onclick: move |_| {
                            let current = read_selected();
                            let new_mode = mode().toggle();
                            write_categories(current, new_mode);
                        },
                        { mode().label() }
                    }
                }
                if !selected.is_empty() {
                    button {
                        class: "clear-btn",
                        onclick: move |_| {
                            write_categories(vec![], mode());
                        },
                        "\u{00d7}"
                    }
                }
            }

            div { class: "flex flex-wrap gap-1 flex-center",
                for cat in MechanicalCategory::all().iter() {
                    {
                        let cat_str = cat.to_string();
                        let display = cat.display_name().to_lowercase();
                        let is_selected = selected.contains(&cat_str);
                        rsx! {
                            div {
                                class: if is_selected { "chip selected" } else { "chip" },
                                onclick: move |_| {
                                    let mut current = read_selected();
                                    let key = cat.to_string();
                                    if current.contains(&key) {
                                        current.retain(|s| s != &key);
                                    } else {
                                        current.push(key);
                                    }
                                    write_categories(current, mode());
                                },
                                { display }
                            }
                        }
                    }
                }
            }

            // ── category excludes ─────────────────────────────────
            div { class: "label-row mt-2",
                label { class: "label-xs", "category excludes" }
                if !excluded.is_empty() {
                    button {
                        class: "clear-btn",
                        onclick: move |_| {
                            write_excluded_cats(vec![]);
                        },
                        "\u{00d7}"
                    }
                }
            }

            div { class: "flex flex-wrap gap-1 flex-center",
                for cat in MechanicalCategory::all().iter() {
                    {
                        let cat_str = cat.to_string();
                        let display = cat.display_name().to_lowercase();
                        let is_excluded = excluded.contains(&cat_str);
                        rsx! {
                            div {
                                class: if is_excluded { "chip selected" } else { "chip" },
                                onclick: move |_| {
                                    let mut current = read_excluded();
                                    let key = cat.to_string();
                                    if current.contains(&key) {
                                        current.retain(|s| s != &key);
                                    } else {
                                        current.push(key);
                                    }
                                    write_excluded_cats(current);
                                },
                                { display }
                            }
                        }
                    }
                }
            }
        }
    }
}
