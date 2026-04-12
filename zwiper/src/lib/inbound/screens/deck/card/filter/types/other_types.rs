//! Other type-line filter sub-component (search-based type selection).

use super::super::{
    deck_cards::{DeckCards, extract_type_words},
    match_mode::MatchMode,
};
use crate::outbound::client::{card::get_card_types::ClientGetCardTypes, ZwipeClient};
use dioxus::prelude::*;
use zwipe_core::domain::card::search_card::card_filter::builder::CardFilterBuilder;
use zwipe::inbound::http::ApiError;
use zwipe_core::domain::auth::models::session::Session;

fn read_other_types(fb: &CardFilterBuilder, mode: MatchMode) -> Vec<String> {
    match mode {
        MatchMode::Any => fb.type_line_contains_any().map(|v| v.to_vec()).unwrap_or_default(),
        MatchMode::All => fb.type_line_contains_all().map(|v| v.to_vec()).unwrap_or_default(),
    }
}

fn write_other_types(fb: &mut CardFilterBuilder, mode: MatchMode, values: Vec<String>) {
    fb.unset_type_line_contains_any();
    fb.unset_type_line_contains_all();
    if !values.is_empty() {
        match mode {
            MatchMode::Any => { fb.set_type_line_contains_any(values); }
            MatchMode::All => { fb.set_type_line_contains_all(values); }
        }
    }
}

fn read_excluded(fb: &CardFilterBuilder) -> Vec<String> {
    fb.type_line_excludes_any().map(|v| v.to_vec()).unwrap_or_default()
}

fn write_excluded(fb: &mut CardFilterBuilder, values: Vec<String>) {
    if values.is_empty() {
        fb.unset_type_line_excludes_any();
    } else {
        fb.set_type_line_excludes_any(values);
    }
}

/// Type line filter with separate include and exclude sections.
#[component]
pub(crate) fn OtherTypes() -> Element {
    let session: Signal<Option<Session>> = use_context();
    let client: Signal<ZwipeClient> = use_context();
    let mut filter_builder: Signal<CardFilterBuilder> = use_context();
    let deck_ctx: Option<DeckCards> = try_use_context();

    let all_card_types: Resource<Result<Vec<String>, ApiError>> =
        use_resource(move || async move {
            if let Some(dc) = deck_ctx {
                return Ok(extract_type_words(&dc.0()));
            }
            let Some(session) = session() else {
                return Err(ApiError::Unauthorized("session expired".to_string()));
            };
            client().get_card_types(&session).await
        });

    let mut search_query = use_signal(String::new);
    let mut is_typing = use_signal(|| false);
    let mut excludes_search = use_signal(String::new);
    let mut excludes_is_typing = use_signal(|| false);

    let filter_reset: Signal<u32> = use_context();

    let mut other_type_mode = use_signal(|| {
        if filter_builder().type_line_contains_all().is_some() {
            MatchMode::All
        } else {
            MatchMode::Any
        }
    });

    use_effect(move || {
        let _ = filter_reset();
        search_query.set(String::new());
        is_typing.set(false);
        excludes_search.set(String::new());
        excludes_is_typing.set(false);
    });

    let selected_other_types = read_other_types(&filter_builder(), other_type_mode());
    let excluded_other_types = read_excluded(&filter_builder());

    rsx! {
        // ── type line includes ────────────────────────────────────
        div { class: "label-row mt-2",
            label { class: "label-xs", r#for: "other-type", "type line include" }
            button {
                class: "clear-btn",
                onclick: move |_| {
                    let new_mode = other_type_mode().toggle();
                    let current = read_other_types(&filter_builder(), other_type_mode());
                    write_other_types(&mut filter_builder.write(), new_mode, current);
                    other_type_mode.set(new_mode);
                },
                "{other_type_mode().label()}"
            }
            if !selected_other_types.is_empty() {
                button {
                    class: "clear-btn",
                    onclick: move |_| {
                        write_other_types(&mut filter_builder.write(), other_type_mode(), vec![]);
                        search_query.set(String::new());
                    },
                    "×"
                }
            }
        }

        if !selected_other_types.is_empty() {
            div { class: "flex flex-wrap gap-1 mb-1",
                for selected in selected_other_types.iter().cloned() {
                    div { class: "chip selected flex items-center gap-05",
                        "{selected}"
                        button { class: "chip-remove",
                            onclick: move |_| {
                                let mode = other_type_mode();
                                let new_types: Vec<String> = read_other_types(&filter_builder(), mode)
                                    .into_iter()
                                    .filter(|t| *t != selected)
                                    .collect();
                                write_other_types(&mut filter_builder.write(), mode, new_types);
                            },
                            "×"
                        }
                    }
                }
            }
        }

        if !search_query().is_empty() {
            if let Some(Ok(all_types)) = all_card_types.read().as_ref() {
                {
                    let results: Vec<String> = all_types
                        .iter()
                        .filter(|t| {
                            let lower = t.to_lowercase();
                            !selected_other_types.contains(&lower)
                                && !excluded_other_types.contains(&lower)
                                && lower.contains(&search_query().to_lowercase())
                        })
                        .take(8)
                        .map(|x| x.to_lowercase())
                        .collect();

                    if !results.is_empty() {
                        rsx! {
                            div { class: "flex flex-wrap gap-1 mb-1",
                                for t in results {
                                    div { class: "chip-unselected",
                                        onclick: move |_| {
                                            let mode = other_type_mode();
                                            let mut current = read_other_types(&filter_builder(), mode);
                                            current.push(t.clone());
                                            write_other_types(&mut filter_builder.write(), mode, current);
                                            is_typing.set(false);
                                        },
                                        "{t}"
                                    }
                                }
                            }
                        }
                    } else {
                        rsx! {}
                    }
                }
            }
        }

        input { class: "input input-compact",
            id: "other-type-search",
            placeholder: "type to search",
            value: "{search_query()}",
            r#type: "text",
            autocapitalize: "none",
            spellcheck: "false",
            oninput: move |event| {
                is_typing.set(true);
                search_query.set(event.value());
            },
            onblur: move |_| {
                is_typing.set(false);
            }
        }

        // ── type line excludes ────────────────────────────────────
        div { class: "label-row mt-2",
            label { class: "label-xs", r#for: "type-excludes-search", "type line exclude" }
            if !excluded_other_types.is_empty() {
                button {
                    class: "clear-btn",
                    onclick: move |_| {
                        write_excluded(&mut filter_builder.write(), vec![]);
                        excludes_search.set(String::new());
                    },
                    "×"
                }
            }
        }

        if !excluded_other_types.is_empty() {
            div { class: "flex flex-wrap gap-1 mb-1",
                for excluded_type in excluded_other_types.iter().cloned() {
                    div { class: "chip selected flex items-center gap-05",
                        "{excluded_type}"
                        button { class: "chip-remove",
                            onclick: move |_| {
                                let new_excluded: Vec<String> = read_excluded(&filter_builder())
                                    .into_iter()
                                    .filter(|t| *t != excluded_type)
                                    .collect();
                                write_excluded(&mut filter_builder.write(), new_excluded);
                            },
                            "×"
                        }
                    }
                }
            }
        }

        if !excludes_search().is_empty() {
            if let Some(Ok(all_types)) = all_card_types.read().as_ref() {
                {
                    let results: Vec<String> = all_types
                        .iter()
                        .filter(|t| {
                            let lower = t.to_lowercase();
                            !selected_other_types.contains(&lower)
                                && !excluded_other_types.contains(&lower)
                                && lower.contains(&excludes_search().to_lowercase())
                        })
                        .take(8)
                        .map(|x| x.to_lowercase())
                        .collect();

                    if !results.is_empty() {
                        rsx! {
                            div { class: "flex flex-wrap gap-1 mb-1",
                                for t in results {
                                    div { class: "chip-unselected",
                                        onclick: move |_| {
                                            let mut current = read_excluded(&filter_builder());
                                            current.push(t.clone());
                                            write_excluded(&mut filter_builder.write(), current);
                                            excludes_is_typing.set(false);
                                        },
                                        "{t}"
                                    }
                                }
                            }
                        }
                    } else {
                        rsx! {}
                    }
                }
            }
        }

        input { class: "input input-compact",
            id: "type-excludes-search",
            placeholder: "type to search",
            value: "{excludes_search()}",
            r#type: "text",
            autocapitalize: "none",
            spellcheck: "false",
            oninput: move |event| {
                excludes_is_typing.set(true);
                excludes_search.set(event.value());
            },
            onblur: move |_| {
                excludes_is_typing.set(false);
            }
        }
    }
}
