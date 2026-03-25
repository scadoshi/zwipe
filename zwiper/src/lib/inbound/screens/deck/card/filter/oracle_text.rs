//! Oracle text filter component.
//!
//! Provides oracle text contains (free text), oracle words contains (chip multi-select),
//! and keywords contains (chip multi-select), each with any/all matching toggles.

use super::{
    deck_cards::{DeckCards, extract_keywords, extract_oracle_words},
    match_mode::MatchMode,
};
use crate::outbound::client::{
    card::{get_keywords::ClientGetKeywords, get_oracle_words::ClientGetOracleWords},
    ZwipeClient,
};
use dioxus::prelude::*;
use zwipe::{
    domain::{
        auth::models::session::Session,
        card::models::search_card::card_filter::builder::CardFilterBuilder,
    },
    inbound::http::ApiError,
};

/// Read selected oracle words from the filter builder based on current mode.
fn read_oracle_words(fb: &CardFilterBuilder, mode: MatchMode) -> Vec<String> {
    match mode {
        MatchMode::Any => fb
            .oracle_text_contains_any()
            .map(|v| v.to_vec())
            .unwrap_or_default(),
        MatchMode::All => fb
            .oracle_text_contains_all()
            .map(|v| v.to_vec())
            .unwrap_or_default(),
    }
}

/// Write oracle words to the filter builder based on current mode.
fn write_oracle_words(fb: &mut CardFilterBuilder, mode: MatchMode, values: Vec<String>) {
    fb.unset_oracle_text_contains_any();
    fb.unset_oracle_text_contains_all();
    if !values.is_empty() {
        match mode {
            MatchMode::Any => { fb.set_oracle_text_contains_any(values); }
            MatchMode::All => { fb.set_oracle_text_contains_all(values); }
        }
    }
}

/// Read selected keywords from the filter builder based on current mode.
fn read_keywords(fb: &CardFilterBuilder, mode: MatchMode) -> Vec<String> {
    match mode {
        MatchMode::Any => fb
            .keywords_contains_any()
            .map(|v| v.to_vec())
            .unwrap_or_default(),
        MatchMode::All => fb
            .keywords_contains_all()
            .map(|v| v.to_vec())
            .unwrap_or_default(),
    }
}

/// Write keywords to the filter builder based on current mode.
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

/// Filter component for oracle text search, oracle words, and keywords.
#[component]
pub fn OracleText() -> Element {
    let mut filter_builder: Signal<CardFilterBuilder> = use_context();

    let session: Signal<Option<Session>> = use_context();
    let client: Signal<ZwipeClient> = use_context();
    let filter_reset: Signal<u32> = use_context();
    let deck_ctx: Option<DeckCards> = try_use_context();

    // Fetch oracle words from deck context (if present) or server
    let all_oracle_words: Resource<Result<Vec<String>, ApiError>> =
        use_resource(move || async move {
            if let Some(dc) = deck_ctx {
                return Ok(extract_oracle_words(&dc.0()));
            }
            let Some(session) = session() else {
                return Err(ApiError::Unauthorized("session expired".to_string()));
            };
            client().get_oracle_words(&session).await
        });

    // Fetch keywords from deck context (if present) or server
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

    // Search state for each chip section
    let mut oracle_words_search = use_signal(String::new);
    let mut keywords_search = use_signal(String::new);

    // Match mode toggles (any vs all)
    let mut oracle_words_mode = use_signal(|| {
        if filter_builder().oracle_text_contains_all().is_some() {
            MatchMode::All
        } else {
            MatchMode::Any
        }
    });
    let mut keywords_mode = use_signal(|| {
        if filter_builder().keywords_contains_all().is_some() {
            MatchMode::All
        } else {
            MatchMode::Any
        }
    });

    // Clear search queries when filter applied
    use_effect(move || {
        let _ = filter_reset();
        oracle_words_search.set(String::new());
        keywords_search.set(String::new());
    });

    // Read selected values for rendering
    let selected_oracle_words = read_oracle_words(&filter_builder(), oracle_words_mode());
    let selected_keywords = read_keywords(&filter_builder(), keywords_mode());

    let oracle_text_value = filter_builder()
        .oracle_text_contains()
        .unwrap_or("")
        .to_string();

    rsx! {
        div { class: "flex-col gap-half",
            // ── oracle text contains ─────────────────────────────────
            div { class: "label-row",
                label { class: "label-xs", r#for: "oracle-text-contains", "oracle text contains" }
                if filter_builder().oracle_text_contains().is_some() {
                    button {
                        class: "clear-btn",
                        onclick: move |_| {
                            filter_builder.write().unset_oracle_text_contains();
                        },
                        "×"
                    }
                }
            }
            input { class: "input input-compact",
                id: "oracle-text-contains",
                placeholder: "oracle text contains",
                value: oracle_text_value,
                r#type: "text",
                autocapitalize: "none",
                spellcheck: "false",
                oninput: move |event| {
                    filter_builder.write().set_oracle_text_contains(event.value());
                }
            }

            // ── oracle words contains (any/all) ──────────────────────
            div { class: "label-row",
                label { class: "label-xs", r#for: "oracle-words-search", "oracle words contains" }
                button {
                    class: "clear-btn",
                    onclick: move |_| {
                        let new_mode = oracle_words_mode().toggle();
                        let current = read_oracle_words(&filter_builder(), oracle_words_mode());
                        write_oracle_words(&mut filter_builder.write(), new_mode, current);
                        oracle_words_mode.set(new_mode);
                    },
                    "{oracle_words_mode().label()}"
                }
                if !selected_oracle_words.is_empty() {
                    button {
                        class: "clear-btn",
                        onclick: move |_| {
                            write_oracle_words(&mut filter_builder.write(), oracle_words_mode(), vec![]);
                            oracle_words_search.set(String::new());
                        },
                        "×"
                    }
                }
            }

            if !selected_oracle_words.is_empty() {
                div { class: "flex flex-wrap gap-1 mb-1",
                    for word in selected_oracle_words.iter().cloned() {
                        div { class: "chip flex items-center gap-05",
                            "{word}"
                            button { class: "chip-remove",
                                onclick: move |_| {
                                    let mode = oracle_words_mode();
                                    let new_words: Vec<String> = read_oracle_words(&filter_builder(), mode)
                                        .into_iter()
                                        .filter(|w| *w != word)
                                        .collect();
                                    write_oracle_words(&mut filter_builder.write(), mode, new_words);
                                },
                                "×"
                            }
                        }
                    }
                }
            }

            if !oracle_words_search().is_empty() {
                if let Some(Ok(words)) = all_oracle_words.read().as_ref() {
                    {
                        let query = oracle_words_search().to_lowercase();
                        let already = read_oracle_words(&filter_builder(), oracle_words_mode());

                        let results: Vec<String> = words
                            .iter()
                            .filter(|w| {
                                w.to_lowercase().contains(&query)
                                && !already.contains(w)
                            })
                            .take(8)
                            .cloned()
                            .collect();

                        if !results.is_empty() {
                            rsx! {
                                div { class: "flex flex-wrap gap-1 mb-1",
                                    for word in results {
                                        div { class: "chip-unselected",
                                            onclick: move |_| {
                                                let mode = oracle_words_mode();
                                                let mut current = read_oracle_words(&filter_builder(), mode);
                                                current.push(word.clone());
                                                write_oracle_words(&mut filter_builder.write(), mode, current);
                                            },
                                            "{word}"
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
                id: "oracle-words-search",
                placeholder: "type to search",
                value: "{oracle_words_search()}",
                r#type: "text",
                autocapitalize: "none",
                spellcheck: "false",
                oninput: move |event| {
                    oracle_words_search.set(event.value());
                }
            }

            // ── keywords contains (any/all) ──────────────────────────
            div { class: "label-row",
                label { class: "label-xs", r#for: "keyword-search", "keywords contains" }
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
                        "×"
                    }
                }
            }

            if !selected_keywords.is_empty() {
                div { class: "flex flex-wrap gap-1 mb-1",
                    for keyword in selected_keywords.iter().cloned() {
                        div { class: "chip flex items-center gap-05",
                            "{keyword}"
                            button { class: "chip-remove",
                                onclick: move |_| {
                                    let mode = keywords_mode();
                                    let new_keywords: Vec<String> = read_keywords(&filter_builder(), mode)
                                        .into_iter()
                                        .filter(|k| *k != keyword)
                                        .collect();
                                    write_keywords(&mut filter_builder.write(), mode, new_keywords);
                                },
                                "×"
                            }
                        }
                    }
                }
            }

            if !keywords_search().is_empty() {
                if let Some(Ok(keywords)) = all_keywords.read().as_ref() {
                    {
                        let query = keywords_search().to_lowercase();
                        let already = read_keywords(&filter_builder(), keywords_mode());

                        let results: Vec<String> = keywords
                            .iter()
                            .filter(|k| {
                                k.to_lowercase().contains(&query)
                                && !already.contains(k)
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
                                            "{keyword}"
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
        }
    }
}
