use crate::outbound::client::{card::get_sets::ClientGetSets, ZwipeClient};
use dioxus::prelude::*;
use zwipe::{
    domain::{
        auth::models::session::Session,
        card::models::search_card::card_filter::builder::CardFilterBuilder,
    },
    inbound::http::ApiError,
};

#[component]
pub fn Set() -> Element {
    let session: Signal<Option<Session>> = use_context();
    let client: Signal<ZwipeClient> = use_context();

    let mut filter_builder: Signal<CardFilterBuilder> = use_context();

    let all_sets: Resource<Result<Vec<String>, ApiError>> = use_resource(move || async move {
        let Some(session) = session() else {
            return Err(ApiError::Unauthorized("session expired".to_string()));
        };
        client().get_sets(&session).await
    });

    let mut search_query = use_signal(String::new);
    let mut is_typing = use_signal(|| false);

    // Get reset signal from parent (add.rs) to clear search on apply
    let filter_reset: Signal<u32> = use_context();

    // Clear search query when filter is applied (keeps this effect - not a sync effect)
    use_effect(move || {
        let _ = filter_reset(); // Subscribe to changes
        search_query.set(String::new());
        is_typing.set(false);
    });

    // Read selected sets directly from filter_builder
    let selected_sets = filter_builder()
        .set_equals_any()
        .map(|v| v.to_vec())
        .unwrap_or_default();

    rsx! {
        div { class: "flex-col gap-half",
            div { class: "label-row",
                label { class: "label-xs", r#for: "set-search", "set equals any" }
                if !selected_sets.is_empty() {
                    button {
                        class: "clear-btn",
                        onclick: move |_| {
                            filter_builder.write().unset_set_equals_any();
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
                        div { class: "chip flex items-center gap-05",
                            "{set}"
                            button { class: "chip-remove",
                                onclick: move |_| {
                                    // Remove from filter_builder directly
                                    let current = filter_builder()
                                        .set_equals_any()
                                        .map(|v| v.to_vec())
                                        .unwrap_or_default();
                                    let new_sets: Vec<String> = current
                                        .into_iter()
                                        .filter(|s| *s != set)
                                        .collect();
                                    if new_sets.is_empty() {
                                        filter_builder.write().unset_set_equals_any();
                                    } else {
                                        filter_builder.write().set_set_equals_any(new_sets);
                                    }
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
                                                // Add to filter_builder directly
                                                let mut current = filter_builder()
                                                    .set_equals_any()
                                                    .map(|v| v.to_vec())
                                                    .unwrap_or_default();
                                                current.push(set.clone());
                                                filter_builder.write().set_set_equals_any(current);
                                                is_typing.set(false);
                                            },
                                            "{set}"
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
        }
    }
}
