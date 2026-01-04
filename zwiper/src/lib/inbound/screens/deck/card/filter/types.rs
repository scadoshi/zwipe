use crate::outbound::client::{card::get_card_types::ClientGetCardTypes, ZwipeClient};
use dioxus::prelude::*;
use zwipe::{
    domain::{
        auth::models::session::Session,
        card::models::search_card::{
            card_filter::builder::CardFilterBuilder,
            card_type::{CardType, WithCardTypes},
        },
    },
    inbound::http::ApiError,
};

#[component]
pub fn Types() -> Element {
    let session: Signal<Option<Session>> = use_context();
    let client: Signal<ZwipeClient> = use_context();

    let mut filter_builder: Signal<CardFilterBuilder> = use_context();

    let all_card_types: Resource<Result<Vec<String>, ApiError>> =
        use_resource(move || async move {
            // Don't call session.upkeep here - use_resource re-runs frequently
            // The interval-based upkeep in Bouncer handles session refresh
            let Some(sesh) = session() else {
                return Err(ApiError::Unauthorized("session expired".to_string()));
            };
            client().get_card_types(&sesh).await
        });

    let mut selected_other_types: Signal<Vec<String>> = use_signal(|| {
        filter_builder()
            .type_line_contains_any()
            .map(|v| v.to_vec())
            .unwrap_or_default()
    });
    let mut search_query = use_signal(String::new);
    let mut is_typing = use_signal(|| false);

    // Get reset signal from parent (add.rs) to clear search on apply
    let filter_reset: Signal<u32> = use_context();

    // Clear search query when filter is applied
    use_effect(move || {
        let _ = filter_reset(); // Subscribe to changes
        search_query.set(String::new());
        is_typing.set(false);
    });

    // Sync local signal TO filter_builder (only if changed)
    use_effect(move || {
        let new_val = selected_other_types();
        let current = filter_builder().type_line_contains_any().map(|v| v.to_vec());
        if current.as_ref() != Some(&new_val) {
            filter_builder.write().set_type_line_contains_any(new_val);
        }
    });

    // Sync FROM filter_builder (handles clear_all)
    use_effect(move || {
        if filter_builder().type_line_contains_any().is_none() {
            selected_other_types.set(Vec::new());
            if !is_typing() {
                search_query.set(String::new());
            }
        }
    });

    rsx! {
        div { class: "flex-col gap-half",
            div { class: "label-row",
                label { class: "label-xs", r#for: "card-type", "basic types" }
                if filter_builder().card_type_contains_any().is_some() {
                    button {
                        class: "clear-btn",
                        onclick: move |_| {
                            filter_builder.write().unset_card_type_contains_any();
                        },
                        "×"
                    }
                }
            }
            div { class: "flex flex-wrap gap-1 mb-1 flex-center",
                for card_type in Vec::with_all_card_types() {
                    div { class: if let Some(card_type_contains_any) = filter_builder().card_type_contains_any() {
                            if card_type_contains_any.contains(&card_type) {
                                "chip selected"
                            } else { "chip" }
                        } else { "chip" },
                        onclick: move |_| {
                            let mut new: Vec<CardType> = Vec::new();
                            if let Some(selected) = filter_builder().card_type_contains_any() {
                                new = selected.to_vec();
                                if selected.contains(&card_type) {
                                    new.retain(|x| x != &card_type);
                                } else {
                                    new.push(card_type);
                                }
                            } else {
                                new.push(card_type);
                            }
                            filter_builder.write().set_card_type_contains_any(new);
                        },
                        "{card_type}"
                    }
                }
            }

            div { class: "label-row",
                label { class: "label-xs", r#for: "other-type", "other types" }
                if !selected_other_types.read().is_empty() {
                    button {
                        class: "clear-btn",
                        onclick: move |_| {
                            selected_other_types.set(Vec::new());
                            search_query.set(String::new());
                        },
                        "×"
                    }
                }
            }

            // Selected types (filled chips with remove button)
            if !selected_other_types.read().is_empty() {
                div { class: "flex flex-wrap gap-1 mb-1",
                    for selected in selected_other_types.iter().map(|x| x.clone()) {
                        div { class: "chip flex items-center gap-05",
                            "{selected}"
                            button { class: "chip-remove",
                                onclick: move |_| {
                                    selected_other_types.write().retain(|x| x != &selected);
                                },
                                "×"
                            }
                        }
                    }
                }
            }

            // Search result bubbles (above input, click to add)
            if !search_query().is_empty() {
                if let Some(Ok(all_types)) = all_card_types.read().as_ref() {
                    {
                        let selected = selected_other_types();
                        let results: Vec<String> = all_types
                            .iter()
                            .filter(|t| {
                                let lower = t.to_lowercase();
                                !selected.contains(&lower)
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
                                                selected_other_types.write().push(t.clone());
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

            // Search input (at the bottom)
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
        }
    }
}
