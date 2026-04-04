//! Oracle words chip multi-select component.

use super::super::{
    deck_cards::{DeckCards, extract_oracle_words},
    match_mode::MatchMode,
};
use crate::outbound::client::{card::get_oracle_words::ClientGetOracleWords, ZwipeClient};
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

/// Oracle words chip multi-select with any/all match mode toggle.
#[component]
pub(crate) fn OracleWords() -> Element {
    let mut filter_builder: Signal<CardFilterBuilder> = use_context();
    let session: Signal<Option<Session>> = use_context();
    let client: Signal<ZwipeClient> = use_context();
    let filter_reset: Signal<u32> = use_context();
    let deck_ctx: Option<DeckCards> = try_use_context();

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

    let mut oracle_words_search = use_signal(String::new);

    let mut oracle_words_mode = use_signal(|| {
        if filter_builder().oracle_text_contains_all().is_some() {
            MatchMode::All
        } else {
            MatchMode::Any
        }
    });

    use_effect(move || {
        let _ = filter_reset();
        oracle_words_search.set(String::new());
    });

    let selected_oracle_words = read_oracle_words(&filter_builder(), oracle_words_mode());

    rsx! {
        div { class: "label-row mt-2",
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
                    "\u{00d7}"
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
                            "\u{00d7}"
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
    }
}
