//! Keywords chip multi-select component.

use super::super::{
    deck_cards::{DeckCards, extract_keywords},
    match_mode::MatchMode,
};
use crate::outbound::client::{card::get_keywords::ClientGetKeywords, ZwipeClient};
use dioxus::prelude::*;
use zwipe_core::domain::card::search_card::card_filter::builder::CardFilterBuilder;
use zwipe::inbound::http::ApiError;
use zwipe_core::domain::auth::models::session::Session;

fn read_keywords(fb: &CardFilterBuilder, mode: MatchMode) -> Vec<String> {
    match mode {
        MatchMode::Any => fb.keywords_contains_any().map(|v| v.to_vec()).unwrap_or_default(),
        MatchMode::All => fb.keywords_contains_all().map(|v| v.to_vec()).unwrap_or_default(),
    }
}

fn write_keywords(fb: &mut CardFilterBuilder, mode: MatchMode, values: Vec<String>) {
    fb.unset_keywords_contains_any();
    fb.unset_keywords_contains_all();
    if !values.is_empty() {
        match mode {
            MatchMode::Any => { fb.set_keywords_contains_any(values); }
            MatchMode::All => { fb.set_keywords_contains_all(values); }
        }
    }
}

fn read_excluded(fb: &CardFilterBuilder) -> Vec<String> {
    fb.keywords_excludes().map(|v| v.to_vec()).unwrap_or_default()
}

fn write_excluded(fb: &mut CardFilterBuilder, values: Vec<String>) {
    if values.is_empty() {
        fb.unset_keywords_excludes();
    } else {
        fb.set_keywords_excludes(values);
    }
}

/// Keywords chip multi-select with any/all match mode toggle.
/// Separate include and exclude sections, each with its own search.
#[component]
pub(crate) fn Keywords() -> Element {
    let mut filter_builder: Signal<CardFilterBuilder> = use_context();
    let session: Signal<Option<Session>> = use_context();
    let client: Signal<ZwipeClient> = use_context();
    let filter_reset: Signal<u32> = use_context();
    let deck_ctx: Option<DeckCards> = try_use_context();

    let all_keywords: Resource<Result<Vec<String>, ApiError>> =
        use_resource(move || async move {
            if let Some(dc) = deck_ctx {
                return Ok(extract_keywords(&dc.0()));
            }
            let Some(session) = session() else {
                return Err(ApiError::Unauthorized("session expired".to_string()));
            };
            client().get_keywords(&session).await
        });

    let mut keywords_search = use_signal(String::new);
    let mut excludes_search = use_signal(String::new);

    let mut keywords_mode = use_signal(|| {
        if filter_builder().keywords_contains_all().is_some() {
            MatchMode::All
        } else {
            MatchMode::Any
        }
    });

    use_effect(move || {
        let _ = filter_reset();
        keywords_search.set(String::new());
        excludes_search.set(String::new());
    });

    let selected_keywords = read_keywords(&filter_builder(), keywords_mode());
    let excluded_keywords = read_excluded(&filter_builder());

    rsx! {
        // ── includes ──────────────────────────────────────────────
        div { class: "label-row mt-2",
            label { class: "label-xs", r#for: "keyword-search", "keywords include" }
            button {
                class: "clear-btn",
                onclick: move |_| {
                    let new_mode = keywords_mode().toggle();
                    let current = read_keywords(&filter_builder(), keywords_mode());
                    write_keywords(&mut filter_builder.write(), new_mode, current);
                    keywords_mode.set(new_mode);
                },
                "{keywords_mode().label()}"
            }
            if !selected_keywords.is_empty() {
                button {
                    class: "clear-btn",
                    onclick: move |_| {
                        write_keywords(&mut filter_builder.write(), keywords_mode(), vec![]);
                        keywords_search.set(String::new());
                    },
                    "\u{00d7}"
                }
            }
        }

        if !selected_keywords.is_empty() {
            div { class: "flex flex-wrap gap-1 mb-1",
                for keyword in selected_keywords.iter().cloned() {
                    div { class: "chip selected flex items-center gap-05",
                        {keyword.to_lowercase()}
                        button { class: "chip-remove",
                            onclick: move |_| {
                                let mode = keywords_mode();
                                let new_keywords: Vec<String> = read_keywords(&filter_builder(), mode)
                                    .into_iter()
                                    .filter(|k| *k != keyword)
                                    .collect();
                                write_keywords(&mut filter_builder.write(), mode, new_keywords);
                            },
                            "\u{00d7}"
                        }
                    }
                }
            }
        }

        if !keywords_search().is_empty() {
            if let Some(Ok(keywords)) = all_keywords.read().as_ref() {
                {
                    let query = keywords_search().to_lowercase();
                    let results: Vec<String> = keywords
                        .iter()
                        .filter(|k| {
                            k.to_lowercase().contains(&query)
                            && !selected_keywords.contains(k)
                            && !excluded_keywords.contains(k)
                        })
                        .take(8)
                        .cloned()
                        .collect();

                    if !results.is_empty() {
                        rsx! {
                            div { class: "flex flex-wrap gap-1 mb-1",
                                for keyword in results {
                                    div { class: "chip-unselected",
                                        onclick: move |_| {
                                            let mode = keywords_mode();
                                            let mut current = read_keywords(&filter_builder(), mode);
                                            current.push(keyword.clone());
                                            write_keywords(&mut filter_builder.write(), mode, current);
                                        },
                                        {keyword.to_lowercase()}
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
            id: "keyword-search",
            placeholder: "type to search",
            value: "{keywords_search()}",
            r#type: "text",
            autocapitalize: "none",
            spellcheck: "false",
            oninput: move |event| {
                keywords_search.set(event.value());
            }
        }

        // ── excludes ──────────────────────────────────────────────
        div { class: "label-row mt-2",
            label { class: "label-xs", r#for: "keyword-excludes-search", "keywords exclude" }
            if !excluded_keywords.is_empty() {
                button {
                    class: "clear-btn",
                    onclick: move |_| {
                        write_excluded(&mut filter_builder.write(), vec![]);
                        excludes_search.set(String::new());
                    },
                    "\u{00d7}"
                }
            }
        }

        if !excluded_keywords.is_empty() {
            div { class: "flex flex-wrap gap-1 mb-1",
                for keyword in excluded_keywords.iter().cloned() {
                    div { class: "chip selected flex items-center gap-05",
                        {keyword.to_lowercase()}
                        button { class: "chip-remove",
                            onclick: move |_| {
                                let new_excluded: Vec<String> = read_excluded(&filter_builder())
                                    .into_iter()
                                    .filter(|k| *k != keyword)
                                    .collect();
                                write_excluded(&mut filter_builder.write(), new_excluded);
                            },
                            "\u{00d7}"
                        }
                    }
                }
            }
        }

        if !excludes_search().is_empty() {
            if let Some(Ok(keywords)) = all_keywords.read().as_ref() {
                {
                    let query = excludes_search().to_lowercase();
                    let results: Vec<String> = keywords
                        .iter()
                        .filter(|k| {
                            k.to_lowercase().contains(&query)
                            && !selected_keywords.contains(k)
                            && !excluded_keywords.contains(k)
                        })
                        .take(8)
                        .cloned()
                        .collect();

                    if !results.is_empty() {
                        rsx! {
                            div { class: "flex flex-wrap gap-1 mb-1",
                                for keyword in results {
                                    div { class: "chip-unselected",
                                        onclick: move |_| {
                                            let mut current = read_excluded(&filter_builder());
                                            current.push(keyword.clone());
                                            write_excluded(&mut filter_builder.write(), current);
                                        },
                                        {keyword.to_lowercase()}
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
            id: "keyword-excludes-search",
            placeholder: "type to search",
            value: "{excludes_search()}",
            r#type: "text",
            autocapitalize: "none",
            spellcheck: "false",
            oninput: move |event| {
                excludes_search.set(event.value());
            }
        }
    }
}
