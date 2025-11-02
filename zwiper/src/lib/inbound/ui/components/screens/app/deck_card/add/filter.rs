use crate::{
    inbound::ui::components::{
        auth::session_upkeep::Upkeep,
        interactions::swipe::{
            config::SwipeConfig,
            direction::Direction as Dir,
            screen_offset::{ScreenOffset, ScreenOffsetMethods},
            state::SwipeState,
            Swipeable,
        },
        success_messages::random_success_message,
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
pub fn Filter(
    swipe_state: Signal<SwipeState>,
    deck_id: Uuid,
    card_filter: Signal<SearchCards>,
    cards: Signal<Vec<Card>>,
) -> Element {
    let swipe_config = SwipeConfig {
        navigation_swipes: vec![Dir::Up],
        submission_swipe: None,
        from_main_screen: ScreenOffset::up(),
    };

    let session: Signal<Option<Session>> = use_context();
    let client: Signal<ZwipeClient> = use_context();

    let mut search_error = use_signal(|| None::<String>);
    let mut success_message = use_signal(|| None::<String>);

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
        success_message.set(None);

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
                    success_message.set(Some(random_success_message()));
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
                    )
                }
                Err(e) => search_error.set(Some(e.to_string())),
            }
        });
    };

    rsx! {
        Swipeable { state: swipe_state, config: swipe_config,
            div { class : "form-container",

                h2 { "card filters" }

                form { class : "filter-group",

                    label { r#for : "name-contains", "name contains" }
                    input { class : "filter-input",
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

                    div { class : "basic-card-type-selection",
                        label { r#for : "card-type", "basic types"}

                        div { class: "basic-type-grid",
                            for card_type in Vec::with_all_card_types() {
                                div {
                                    class: if selected_basic_card_types().contains(&card_type) {
                                        "basic-type-box selected"
                                    } else {
                                        "basic-type-box"
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

                    div { class : "other-card-type-selection",
                        label { r#for : "other-type", "other types"}

                        if !selected_other_types().is_empty() {
                            div { class: "selected-types",
                                for selected_type in selected_other_types().iter().map(|x| x.clone()) {
                                    div { class: "type-chip",
                                        "{selected_type}",
                                        button { class: "remove-chip",
                                            onclick: move |_| {
                                                selected_other_types.write().retain(|x| x != &selected_type);
                                            },
                                            "×"
                                        }
                                    }
                                }
                            }
                        }

                        input { class : "form-input",
                            id : "other-type-search",
                            placeholder : "type to search types",
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

                    button { class : "search-cards-button",
                        onclick : move |_| attempt_search_cards(),
                        "search"
                    }

                    if let Some(search_error) = search_error() {
                        div { class : "error", "{search_error}" }
                    }

                    if let Some(success_message) = success_message() {
                        div { class: "success-message", "{success_message}" }
                    }
                }
            }
        }
    }
}
