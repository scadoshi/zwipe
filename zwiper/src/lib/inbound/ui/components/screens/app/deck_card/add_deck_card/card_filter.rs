use crate::{
    inbound::ui::{
        components::{
            auth::session_upkeep::Upkeep,
            interactions::swipe::{config::SwipeConfig, state::SwipeState, Swipeable},
        },
        router::Router,
    },
    outbound::client::{
        card::{get_card_types::ClientGetCardTypes, search_cards::ClientSearchCards},
        ZwipeClient,
    },
};
use dioxus::prelude::*;
use uuid::Uuid;
use zwipe::{
    domain::{
        auth::models::session::Session,
        card::models::{
            search_card::{
                card_type::{CardType, WithCardTypes},
                SearchCards,
            },
            Card,
        },
    },
    inbound::http::ApiError,
};

#[component]
pub fn AddDeckCardFilter(
    card_filter: Signal<SearchCards>,
    cards: Signal<Vec<Card>>,
    deck_id: Uuid,
) -> Element {
    let swipe_config = SwipeConfig::blank();
    let swipe_state = use_signal(|| SwipeState::new());

    let session: Signal<Option<Session>> = use_context();
    let client: Signal<ZwipeClient> = use_context();

    let navigator = use_navigator();

    let mut search_error = use_signal(|| None::<String>);

    let all_card_types: Resource<Result<Vec<String>, ApiError>> =
        use_resource(move || async move {
            session.upkeep(client);
            let Some(sesh) = session() else {
                return Err(ApiError::Unauthorized("session expired".to_string()));
            };
            client().get_card_types(&sesh).await
        });

    let mut selected_basic_card_types: Signal<Vec<CardType>> = use_signal(|| Vec::new());

    let mut other_type_query_string = use_signal(|| String::new());
    let mut selected_other_types: Signal<Vec<String>> = use_signal(|| Vec::new());
    let mut show_other_types_dropdown = use_signal(|| false);

    let mut attempt_search_cards = move || {
        card_filter.write().card_type_contains_any = if selected_basic_card_types().is_empty() {
            None
        } else {
            Some(selected_basic_card_types())
        };

        card_filter.write().type_line_contains_any = if selected_other_types().is_empty() {
            None
        } else {
            Some(selected_other_types())
        };

        if card_filter.read().is_blank() {
            search_error.set(Some("must search something".to_string()));
            return;
        }

        session.upkeep(client);
        let Some(sesh) = session() else {
            search_error.set(Some("session expired".to_string()));
            return;
        };

        spawn(async move {
            match client().search_cards(&card_filter.read(), &sesh).await {
                Ok(cards_from_search) => {
                    search_error.set(None);
                    cards.set(
                        cards_from_search
                            .into_iter()
                            .filter(|card| {
                                card.scryfall_data
                                    .image_uris
                                    .as_ref()
                                    .and_then(|x| x.large.as_ref())
                                    .is_some()
                            })
                            .collect(),
                    );
                    // debug remove this later
                    tracing::debug!("{} cards found", { cards.len() });
                    tracing::debug!("{:#?}", cards);
                    navigator.push(Router::AddDeckCard {
                        deck_id,
                        card_filter,
                        cards,
                    });
                }
                Err(e) => search_error.set(Some(e.to_string())),
            }
        });
    };

    rsx! {
        Swipeable { state: swipe_state, config: swipe_config,
            div { class : "container-sm",

                h2 { class: "text-center mb-2 font-light tracking-wider", "card filters" }

                form { class : "flex-col text-center",

                    label { class: "label", r#for : "name-contains", "name contains" }
                    input { class : "input",
                        id : "name-contains",
                        placeholder : "name contains",
                        value : if let Some(name) = card_filter.read().name_contains.as_deref() {
                            name
                        } else { "" },
                        r#type : "text",
                        autocapitalize : "none",
                        spellcheck : "false",
                        oninput : move |event| {
                            card_filter.write().name_contains = Some(event.value());
                            if card_filter.read().name_contains == Some("".to_string()) {
                                card_filter.write().name_contains = None;
                            }
                        }
                    }

                    div { class : "flex-col text-center",
                        label { class: "label", r#for : "card-type", "basic types"}

                        div { class: "flex flex-wrap gap-1 mb-2 flex-center",
                            for card_type in Vec::with_all_card_types() {
                                div { class: if selected_basic_card_types().contains(&card_type) {
                                        "type-box selected"
                                    } else {
                                        "type-box"
                                    },
                                    onclick: move |_| {
                                        if selected_basic_card_types().contains(&card_type) {
                                            selected_basic_card_types.write().retain(|x| x != &card_type);
                                        } else {
                                            selected_basic_card_types.write().push(card_type.clone());
                                        }
                                    },
                                    "{card_type}"
                                }
                            }
                        }
                    }

                    div { class : "flex-col text-center",
                        label { class: "label", r#for : "other-type", "other types"}

                        if !selected_other_types().is_empty() {
                            div { class: "flex flex-wrap gap-1 mb-2",
                                for selected_type in selected_other_types().iter().map(|x| x.clone()) {
                                    div { class: "type-chip",
                                        "{selected_type}",
                                        button { class: "type-chip-remove",
                                            onclick: move |_| {
                                                selected_other_types.write().retain(|x| x != &selected_type);
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
                            value : "{other_type_query_string}",
                            r#type : "text",
                            autocapitalize : "none",
                            spellcheck : "false",
                            oninput : move |event| {
                                other_type_query_string.set(event.value());
                                show_other_types_dropdown.set(!event.value().is_empty());
                            }
                        }

                        if show_other_types_dropdown() {
                            if let Some(Ok(all_types)) = all_card_types.read().as_ref() {
                                div { class : "dropdown",
                                    for t in all_types
                                        .iter()
                                        .filter(|t| !selected_other_types().contains(t))
                                        .filter(|t| t.to_lowercase().contains(&other_type_query_string().to_lowercase()))
                                        .take(5)
                                        .map(|x| x.clone())
                                    {
                                        div { class: "dropdown-item",
                                            onclick : move |_| {
                                                selected_other_types.write().push(t.clone());
                                                other_type_query_string.set(String::new());
                                                show_other_types_dropdown.set(false);
                                            },
                                            "{t}"
                                        }
                                    }
                                }
                            }
                        }
                    }

                    button { class : "btn",
                        onclick : move |_| attempt_search_cards(),
                        "search"
                    }

                    if let Some(search_error) = search_error() {
                        div { class : "message-error", "{search_error}" }
                    }

                    button { class : "btn",
                        onclick : move |_| {
                            navigator.push(Router::AddDeckCard { deck_id, card_filter, cards });
                        },
                        "back"
                    }
                }
            }
        }
    }
}
