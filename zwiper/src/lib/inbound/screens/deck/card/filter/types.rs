use crate::{
    inbound::components::auth::{bouncer::Bouncer, session_upkeep::Upkeep},
    outbound::client::{ZwipeClient, card::get_card_types::ClientGetCardTypes},
};
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
    let navigator = use_navigator();

    let session: Signal<Option<Session>> = use_context();
    let client: Signal<ZwipeClient> = use_context();

    let mut filter_builder: Signal<CardFilterBuilder> = use_context();

    let all_card_types: Resource<Result<Vec<String>, ApiError>> =
        use_resource(move || async move {
            session.upkeep(client);
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
    let mut other_type_query_string_for_dropdown = use_signal(String::new);
    let mut show_other_type_dropdown = use_signal(|| false);

    use_effect(move || {
        filter_builder
            .write()
            .set_type_line_contains_any(selected_other_types());
    });

    rsx! {
    Bouncer {
        div { class: "fixed top-0 left-0 h-screen flex flex-col items-center overflow-y-auto",
            style: "width: 100vw; justify-content: center;",
            div { class : "container-sm",
                        form { class : "flex-col text-center",
                        label { class: "label", r#for : "card-type", "basic types"}
                            div { class: "flex flex-wrap gap-1 mb-2 flex-center",
                                for card_type in Vec::with_all_card_types() {
                                    div { class: if let Some(card_type_contains_any) = filter_builder().card_type_contains_any() {
                                            if card_type_contains_any.contains(&card_type) {
                                                "type-box selected"
                                            } else { "type-box" }
                                        } else { "type-box" },
                                        onclick: move |_| {
                                            let mut new: Vec<CardType> = Vec::new();
                                            if let Some(selected) = filter_builder().card_type_contains_any() {
                                                new = selected.to_vec();
                                                if selected.contains(&card_type) {
                                                    new.retain(|x| x!= &card_type);
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
                    }

                    div { class : "flex-col text-center",
                        label { class: "label", r#for : "other-type", "other types"}

                        if !selected_other_types.read().is_empty() {
                                div { class: "flex flex-wrap gap-1 mb-2",
                                    for selected in selected_other_types.iter().map(|x| x.clone()) {
                                        div { class: "type-chip",
                                            "{selected}",
                                            button { class: "type-chip-remove",
                                                onclick: move |_| {
                                                    selected_other_types.write().retain(|x| x != &selected);
                                                },
                                                "×"
                                            }
                                        }
                                    }
                                }
                            }


                        input { class : "input",
                            id : "other-type-search",
                            placeholder : "type to search...",
                            value : "{other_type_query_string_for_dropdown()}",
                            r#type : "text",
                            autocapitalize : "none",
                            spellcheck : "false",
                            oninput : move |event| {
                                other_type_query_string_for_dropdown.set(event.value());
                                show_other_type_dropdown.set(!event.value().is_empty());
                            }
                        }

                        if show_other_type_dropdown() {
                            if let Some(Ok(all_types)) = all_card_types.read().as_ref() {
                                div { class : "dropdown",
                                    {
                                        let filter_builder = filter_builder();
                                        let results: Vec<String> = all_types
                                            .iter()
                                            .filter(|t|
                                                    !filter_builder.type_line_contains_any().is_some_and(|v| v.contains(t))
                                                    && t.to_lowercase().contains(&other_type_query_string_for_dropdown().to_lowercase())
                                            )
                                            .take(5)
                                            .map(|x| x.to_lowercase())
                                            .collect();
                                        if results.is_empty() {
                                            rsx! {
                                                div { class : "dropdown-item", "no results" }
                                            }
                                        } else {
                                            rsx! {
                                                for t in results
                                                {
                                                    div { class: "dropdown-item",
                                                        onclick : move |_| {
                                                            selected_other_types.write().push(t.clone());
                                                            other_type_query_string_for_dropdown.set(String::new());
                                                            show_other_type_dropdown.set(false);
                                                        },
                                                        "{t}"
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                    }
                }
            }

            div { class: "util-bar",
                button {
                    class: "util-btn",
                    onclick: move |_| navigator.go_back(),
                    "back"
                }
            }
        }
    }
}

#[component]
pub fn TypesFilterContent() -> Element {
    let session: Signal<Option<Session>> = use_context();
    let client: Signal<ZwipeClient> = use_context();

    let mut filter_builder: Signal<CardFilterBuilder> = use_context();

    let all_card_types: Resource<Result<Vec<String>, ApiError>> =
        use_resource(move || async move {
            session.upkeep(client);
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
    let mut other_type_query_string_for_dropdown = use_signal(String::new);
    let mut show_other_type_dropdown = use_signal(|| false);

    use_effect(move || {
        filter_builder
            .write()
            .set_type_line_contains_any(selected_other_types());
    });

    rsx! {
        div { class: "flex-col gap-half",
            label { class: "label-xs", r#for: "card-type", "basic types" }
            div { class: "flex flex-wrap gap-1 mb-1 flex-center",
                for card_type in Vec::with_all_card_types() {
                    div { class: if let Some(card_type_contains_any) = filter_builder().card_type_contains_any() {
                            if card_type_contains_any.contains(&card_type) {
                                "type-box-compact selected"
                            } else { "type-box-compact" }
                        } else { "type-box-compact" },
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

            label { class: "label-xs", r#for: "other-type", "other types" }

            if !selected_other_types.read().is_empty() {
                div { class: "flex flex-wrap gap-1 mb-1",
                    for selected in selected_other_types.iter().map(|x| x.clone()) {
                        div { class: "type-chip",
                            "{selected}",
                            button { class: "type-chip-remove",
                                onclick: move |_| {
                                    selected_other_types.write().retain(|x| x != &selected);
                                },
                                "×"
                            }
                        }
                    }
                }
            }

            input { class: "input input-compact",
                id: "other-type-search",
                placeholder: "type to search...",
                value: "{other_type_query_string_for_dropdown()}",
                r#type: "text",
                autocapitalize: "none",
                spellcheck: "false",
                oninput: move |event| {
                    other_type_query_string_for_dropdown.set(event.value());
                    show_other_type_dropdown.set(!event.value().is_empty());
                }
            }

            if show_other_type_dropdown() {
                if let Some(Ok(all_types)) = all_card_types.read().as_ref() {
                    div { class: "dropdown dropdown-compact",
                        {
                            let filter_builder = filter_builder();
                            let results: Vec<String> = all_types
                                .iter()
                                .filter(|t|
                                        !filter_builder.type_line_contains_any().is_some_and(|v| v.contains(t))
                                        && t.to_lowercase().contains(&other_type_query_string_for_dropdown().to_lowercase())
                                )
                                .take(5)
                                .map(|x| x.to_lowercase())
                                .collect();
                            if results.is_empty() {
                                rsx! {
                                    div { class: "dropdown-item-compact", "no results" }
                                }
                            } else {
                                rsx! {
                                    for t in results {
                                        div { class: "dropdown-item-compact",
                                            onclick: move |_| {
                                                selected_other_types.write().push(t.clone());
                                                other_type_query_string_for_dropdown.set(String::new());
                                                show_other_type_dropdown.set(false);
                                            },
                                            "{t}"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
