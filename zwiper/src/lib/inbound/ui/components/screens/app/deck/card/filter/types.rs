use crate::{
    inbound::ui::components::{
        auth::{bouncer::Bouncer, session_upkeep::Upkeep},
        interactions::swipe::{config::SwipeConfig, state::SwipeState, Swipeable},
    },
    outbound::client::{card::get_card_types::ClientGetCardTypes, ZwipeClient},
};
use dioxus::prelude::*;
use zwipe::{
    domain::{
        auth::models::session::Session,
        card::models::search_card::{
            card_type::{CardType, WithCardTypes},
            SearchCards,
        },
    },
    inbound::http::ApiError,
};

#[component]
pub fn Types() -> Element {
    let swipe_config = SwipeConfig::blank();
    let swipe_state = use_signal(|| SwipeState::new());
    let navigator = use_navigator();

    let session: Signal<Option<Session>> = use_context();
    let client: Signal<ZwipeClient> = use_context();

    let mut filter: Signal<SearchCards> = use_context();

    let all_card_types: Resource<Result<Vec<String>, ApiError>> =
        use_resource(move || async move {
            session.upkeep(client);
            let Some(sesh) = session() else {
                return Err(ApiError::Unauthorized("session expired".to_string()));
            };
            client().get_card_types(&sesh).await
        });

    let mut selected_other_types: Signal<Vec<String>> = use_signal(|| Vec::new());
    let mut other_type_query_string_for_dropdown = use_signal(|| String::new());
    let mut show_other_type_dropdown = use_signal(|| false);

    use_effect(move || {
        if selected_other_types().is_empty() {
            filter.write().type_line_contains_any = None;
        } else {
            filter.write().type_line_contains_any = Some(selected_other_types.read().clone());
        }
    });

    rsx! {
        Bouncer {
            Swipeable { state: swipe_state, config: swipe_config,
                div { class : "flex-col text-center",
                    label { class: "label", r#for : "card-type", "basic types"}
                        div { class: "flex flex-wrap gap-1 mb-2 flex-center",
                            for card_type in Vec::with_all_card_types() {
                                div { class: if let Some(card_type_contains_any) = &filter.read().card_type_contains_any {
                                        if card_type_contains_any.contains(&card_type) {
                                            "type-box selected"
                                        } else { "type-box" }
                                    } else { "type-box" },
                                    onclick: move |_| {
                                        let mut new: Vec<CardType> = Vec::new();
                                        if let Some(selected) = &filter.read().card_type_contains_any {
                                            new = selected.clone();
                                            if selected.contains(&card_type) {
                                                new.retain(|x| x!= &card_type);
                                            } else {
                                                new.push(card_type.clone());
                                            }
                                        } else {
                                            new.push(card_type.clone());
                                        }
                                        if new.is_empty() {
                                            filter.write().card_type_contains_any = None
                                        } else {
                                            filter.write().card_type_contains_any = Some(new);
                                        }
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
                            let query_string = event.value();
                            let has_query = !query_string.is_empty();
                            if has_query {
                                other_type_query_string_for_dropdown.set(query_string);
                            } else {
                                other_type_query_string_for_dropdown.set(String::new());
                            }
                            show_other_type_dropdown.set(has_query);
                        }
                    }

                    if show_other_type_dropdown() {
                        if let Some(Ok(all_types)) = all_card_types.read().as_ref() {
                            div { class : "dropdown",
                                for t in all_types
                                    .iter()
                                    .filter(|t| !filter.read().type_line_contains_any.as_ref().unwrap_or(&Vec::new()).contains(t))
                                    .filter(|t| t.to_lowercase().contains(&other_type_query_string_for_dropdown().to_lowercase()))
                                    .take(5)
                                    .map(|x| x.clone())
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

                    button { class : "btn",
                        onclick : move |_| {
                            navigator.go_back();
                        },
                        "back"
                    }
                }
            }
        }
    }
}
