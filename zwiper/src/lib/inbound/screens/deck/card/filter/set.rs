use crate::{
    inbound::components::auth::{bouncer::Bouncer, session_upkeep::Upkeep},
    outbound::client::{ZwipeClient, card::get_sets::ClientGetSets},
};
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
    let navigator = use_navigator();

    let mut filter_builder: Signal<CardFilterBuilder> = use_context();

    let session: Signal<Option<Session>> = use_context();
    let client: Signal<ZwipeClient> = use_context();

    let all_sets: Resource<Result<Vec<String>, ApiError>> = use_resource(move || async move {
        session.upkeep(client);
        let Some(sesh) = session() else {
            return Err(ApiError::Unauthorized("session expired".to_string()));
        };
        client().get_sets(&sesh).await
    });

    let mut set_query_string = use_signal(String::new);
    let mut show_set_dropdown = use_signal(|| false);

    rsx! {
        Bouncer {
            div { class: "fixed top-0 left-0 h-screen flex flex-col items-center overflow-y-auto",
                style: "width: 100vw; justify-content: center;",
                div { class : "container-sm",
                    h2 { class: "text-center mb-2 font-light tracking-wider", "set filter" }

                    form { class: "flex-col text-center",
                        label { class: "label", r#for: "set-search", "sets" }

                        if let Some(selected) = filter_builder().set_equals_any() && !selected.is_empty() {
                                div { class: "flex flex-wrap gap-1 mb-2",
                                    for set in selected.to_vec() {
                                        div { class: "type-chip",
                                            "{set}",
                                            button { class: "type-chip-remove",
                                                onclick: move |_| {
                                                    let current = filter_builder()
                                                        .set_equals_any()
                                                        .map(|s| s.to_vec())
                                                        .unwrap_or_default();

                                                    let mut new = current;
                                                    new.retain(|s| *s != set);

                                                    if new.is_empty() {
                                                        filter_builder.write().unset_set_equals_any();
                                                    } else {
                                                        filter_builder.write().set_set_equals_any(new);
                                                    }
                                                },
                                                "×"
                                            }
                                        }
                                    }
                                }
                        }

                        input { class: "input",
                            id: "set-search",
                            placeholder: "type to search sets...",
                            value: "{set_query_string()}",
                            r#type: "text",
                            autocapitalize: "none",
                            spellcheck: "false",
                            oninput: move |event| {
                                set_query_string.set(event.value());
                                show_set_dropdown.set(!event.value().is_empty());
                            }
                        }

                        if show_set_dropdown() {
                            if let Some(Ok(sets)) = all_sets.read().as_ref() {
                                div { class: "dropdown",
                                    {
                                        let query = set_query_string().to_lowercase();
                                        let already_selected = filter_builder()
                                            .set_equals_any()
                                            .map(|s| s.to_vec())
                                            .unwrap_or_default();

                                        let results: Vec<String> = sets
                                            .iter()
                                            .filter(|s| {
                                                s.to_lowercase().contains(&query)
                                                && !already_selected.contains(s)
                                            })
                                            .take(10)
                                            .cloned()
                                            .collect();

                                        if results.is_empty() {
                                            rsx! {
                                                div { class: "dropdown-item", "no matching sets" }
                                            }
                                        } else {
                                            rsx! {
                                                for set in results {
                                                    div { class: "dropdown-item",
                                                        onclick: move |_| {
                                                            let current = filter_builder()
                                                                .set_equals_any()
                                                                .map(|s| s.to_vec())
                                                                .unwrap_or_default();

                                                            let mut new = current;
                                                            new.push(set.clone());
                                                            filter_builder.write().set_set_equals_any(new);
                                                            set_query_string.set(String::new());
                                                            show_set_dropdown.set(false);
                                                        },
                                                        "{set}"
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            } else if all_sets.read().as_ref().is_some_and(|r| r.is_err()) {
                                div { class: "message-error", "Failed to load sets" }
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
pub fn SetFilterContent() -> Element {
    let session: Signal<Option<Session>> = use_context();
    let client: Signal<ZwipeClient> = use_context();

    let mut filter_builder: Signal<CardFilterBuilder> = use_context();

    let all_sets: Resource<Result<Vec<String>, ApiError>> = use_resource(move || async move {
        session.upkeep(client);
        let Some(sesh) = session() else {
            return Err(ApiError::Unauthorized("session expired".to_string()));
        };
        client().get_sets(&sesh).await
    });

    let mut set_query_string = use_signal(String::new);
    let mut show_set_dropdown = use_signal(|| false);

    rsx! {
        div { class: "flex-col gap-half",
            label { class: "label-xs", r#for: "set-search", "sets" }

            if let Some(selected) = filter_builder().set_equals_any() && !selected.is_empty() {
                div { class: "flex flex-wrap gap-1 mb-1",
                    for set in selected.to_vec() {
                        div { class: "type-chip",
                            "{set}",
                            button { class: "type-chip-remove",
                                onclick: move |_| {
                                    let current = filter_builder()
                                        .set_equals_any()
                                        .map(|s| s.to_vec())
                                        .unwrap_or_default();

                                    let mut new = current;
                                    new.retain(|s| *s != set);

                                    if new.is_empty() {
                                        filter_builder.write().unset_set_equals_any();
                                    } else {
                                        filter_builder.write().set_set_equals_any(new);
                                    }
                                },
                                "×"
                            }
                        }
                    }
                }
            }

            input { class: "input input-compact",
                id: "set-search",
                placeholder: "type to search sets...",
                value: "{set_query_string()}",
                r#type: "text",
                autocapitalize: "none",
                spellcheck: "false",
                oninput: move |event| {
                    set_query_string.set(event.value());
                    show_set_dropdown.set(!event.value().is_empty());
                }
            }

            if show_set_dropdown() {
                if let Some(Ok(sets)) = all_sets.read().as_ref() {
                    div { class: "dropdown dropdown-compact",
                        {
                            let query = set_query_string().to_lowercase();
                            let already_selected = filter_builder()
                                .set_equals_any()
                                .map(|s| s.to_vec())
                                .unwrap_or_default();

                            let results: Vec<String> = sets
                                .iter()
                                .filter(|s| {
                                    s.to_lowercase().contains(&query)
                                    && !already_selected.contains(s)
                                })
                                .take(10)
                                .cloned()
                                .collect();

                            if results.is_empty() {
                                rsx! {
                                    div { class: "dropdown-item-compact", "no matching sets" }
                                }
                            } else {
                                rsx! {
                                    for set in results {
                                        div { class: "dropdown-item-compact",
                                            onclick: move |_| {
                                                let current = filter_builder()
                                                    .set_equals_any()
                                                    .map(|s| s.to_vec())
                                                    .unwrap_or_default();

                                                let mut new = current;
                                                new.push(set.clone());
                                                filter_builder.write().set_set_equals_any(new);
                                                set_query_string.set(String::new());
                                                show_set_dropdown.set(false);
                                            },
                                            "{set}"
                                        }
                                    }
                                }
                            }
                        }
                    }
                } else if all_sets.read().as_ref().is_some_and(|r| r.is_err()) {
                    div { class: "message-error", "Failed to load sets" }
                }
            }
        }
    }
}
