//! Card set filter component.

use super::deck_cards::{DeckCards, extract_sets};
use crate::outbound::client::{ZwipeClient, card::get_sets::ClientGetSets};
use dioxus::prelude::*;
use zwipe::inbound::http::ApiError;
use zwipe_core::domain::card::search_card::card_filter::builder::CardFilterBuilder;

/// Whether the set filter is in include or exclude mode.
#[derive(Debug, Clone, Copy, PartialEq)]
enum IncludeExclude {
    Include,
    Exclude,
}

impl IncludeExclude {
    fn toggle(self) -> Self {
        match self {
            Self::Include => Self::Exclude,
            Self::Exclude => Self::Include,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Include => "Include",
            Self::Exclude => "Exclude",
        }
    }
}

fn read_sets(fb: &CardFilterBuilder, mode: IncludeExclude) -> Vec<String> {
    match mode {
        IncludeExclude::Include => fb.set_equals_any().map(|v| v.to_vec()).unwrap_or_default(),
        IncludeExclude::Exclude => fb
            .set_excludes_any()
            .map(|v| v.to_vec())
            .unwrap_or_default(),
    }
}

fn write_sets(fb: &mut CardFilterBuilder, mode: IncludeExclude, values: Vec<String>) {
    fb.unset_set_equals_any();
    fb.unset_set_excludes_any();
    if !values.is_empty() {
        match mode {
            IncludeExclude::Include => {
                fb.set_set_equals_any(values);
            }
            IncludeExclude::Exclude => {
                fb.set_set_excludes_any(values);
            }
        }
    }
}

/// Filter component for selecting card sets.
#[component]
pub fn Set() -> Element {
    let client: Signal<ZwipeClient> = use_context();

    let mut filter_builder: Signal<CardFilterBuilder> = use_context();
    let deck_ctx: Option<DeckCards> = try_use_context();

    let all_sets: Resource<Result<Vec<String>, ApiError>> = use_resource(move || async move {
        if let Some(dc) = deck_ctx {
            return Ok(extract_sets(&dc.0()));
        }
        client().get_sets().await
    });

    let mut search_query = use_signal(String::new);
    let mut is_typing = use_signal(|| false);

    let filter_reset: Signal<u32> = use_context();

    let mut mode = use_signal(|| {
        if filter_builder().set_excludes_any().is_some() {
            IncludeExclude::Exclude
        } else {
            IncludeExclude::Include
        }
    });

    use_effect(move || {
        let _ = filter_reset();
        search_query.set(String::new());
        is_typing.set(false);
    });

    let selected_sets = read_sets(&filter_builder(), mode());
    let has_selection = !selected_sets.is_empty();

    rsx! {
        div { class: "flex-col gap-half",
            div { class: "label-row mt-2",
                label { class: "label-xs", r#for: "set-search", "Set" }
                if has_selection {
                    button {
                        class: "clear-btn",
                        onclick: move |_| {
                            let new_mode = mode().toggle();
                            let current = read_sets(&filter_builder(), mode());
                            write_sets(&mut filter_builder.write(), new_mode, current);
                            mode.set(new_mode);
                        },
                        "{mode().label()}"
                    }
                }
                if has_selection {
                    button {
                        class: "clear-btn",
                        onclick: move |_| {
                            write_sets(&mut filter_builder.write(), mode(), vec![]);
                            search_query.set(String::new());
                        },
                        "×"
                    }
                }
            }

            // Selected sets (filled chips with remove button)
            if !selected_sets.is_empty() {
                div { class: "flex flex-wrap gap-1 mb-1",
                    for set in selected_sets.iter().cloned() {
                        div {
                            class: match mode() {
                                IncludeExclude::Include => "chip selected flex items-center gap-05",
                                IncludeExclude::Exclude => "chip selected flex items-center gap-05",
                            },
                            {set.clone()}
                            button { class: "chip-remove",
                                onclick: move |_| {
                                    let new_sets: Vec<String> = read_sets(&filter_builder(), mode())
                                        .into_iter()
                                        .filter(|s| *s != set)
                                        .collect();
                                    write_sets(&mut filter_builder.write(), mode(), new_sets);
                                },
                                "×"
                            }
                        }
                    }
                }
            }

            // Search result bubbles (above input, click to add)
            if !search_query().is_empty() {
                if let Some(Ok(sets)) = all_sets.read().as_ref() {
                    {
                        let query = search_query().to_lowercase();
                        let already_selected = selected_sets.as_slice();

                        let results: Vec<String> = sets
                            .iter()
                            .filter(|s| {
                                s.to_lowercase().contains(&query)
                                && !already_selected.contains(s)
                            })
                            .take(5)
                            .cloned()
                            .collect();

                        if !results.is_empty() {
                            rsx! {
                                div { class: "flex flex-wrap gap-1 mb-1",
                                    for set in results {
                                        div { class: "chip-unselected",
                                            onclick: move |_| {
                                                let mut current = read_sets(&filter_builder(), mode());
                                                current.push(set.clone());
                                                write_sets(&mut filter_builder.write(), mode(), current);
                                                is_typing.set(false);
                                            },
                                            {set.clone()}
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

            // Search input (at the bottom)
            input { class: "input input-compact",
                id: "set-search",
                placeholder: "Type to search",
                value: "{search_query()}",
                r#type: "text",
                autocapitalize: "none",
                autocorrect: "off",
                spellcheck: "false",
                oninput: move |event| {
                    is_typing.set(true);
                    search_query.set(event.value());
                },
                onblur: move |_| {
                    is_typing.set(false);
                }
            }
        }
    }
}
