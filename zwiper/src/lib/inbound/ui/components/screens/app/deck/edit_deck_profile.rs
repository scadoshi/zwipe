use std::time::Duration;

use crate::{
    inbound::ui::{
        components::{
            auth::{bouncer::Bouncer, session_upkeep::Upkeep},
            interactions::swipe::{config::SwipeConfig, state::SwipeState, Swipeable},
        },
        router::Router,
    },
    outbound::client::{
        card::{get_card::ClientGetCard, search_cards::ClientSearchCards},
        deck::{
            delete_deck::ClientDeleteDeck, get_deck_profile::ClientGetDeckProfile,
            update_deck_profile::ClientUpdateDeckProfile,
        },
        ZwipeClient,
    },
};
use dioxus::prelude::*;
use tokio::time::sleep;
use uuid::Uuid;
use zwipe::{
    domain::{
        auth::models::session::Session,
        card::models::Card,
        deck::models::deck::{copy_max::CopyMax, deck_profile::DeckProfile},
    },
    inbound::http::{
        handlers::{
            card::search_card::HttpSearchCards, deck::update_deck_profile::HttpUpdateDeckProfile,
        },
        ApiError,
    },
};

#[component]
pub fn EditDeckProfile(deck_id: Uuid) -> Element {
    let swipe_state = use_signal(|| SwipeState::new());
    let swipe_config = SwipeConfig::blank();

    let navigator = use_navigator();

    let session: Signal<Option<Session>> = use_context();
    let client: Signal<ZwipeClient> = use_context();

    let deck_profile_resource: Resource<Result<DeckProfile, ApiError>> =
        use_resource(move || async move {
            session.upkeep(client);
            let Some(sesh) = session() else {
                return Err(ApiError::Unauthorized("session expired".to_string()));
            };

            client().get_deck_profile(deck_id, &sesh).await
        });

    let commander_resource: Resource<Result<Option<Card>, ApiError>> =
        use_resource(move || async move {
            let Some(Ok(profile)) = &*deck_profile_resource.read() else {
                return Ok(None);
            };

            let Some(commander_id) = profile.commander_id else {
                return Ok(None);
            };

            let Some(sesh) = session() else {
                return Err(ApiError::Unauthorized("session expired".to_string()));
            };

            client()
                .get_card(&commander_id, &sesh)
                .await
                .map(|value| Some(value))
        });

    // form
    let mut deck_name: Signal<String> = use_signal(|| String::new());
    let mut commander: Signal<Option<Card>> = use_signal(|| None);
    let mut commander_display = use_signal(|| String::new());
    let mut copy_max: Signal<Option<CopyMax>> = use_signal(|| None);

    let mut original_deck_name: Signal<String> = use_signal(|| String::new());
    let mut original_commander: Signal<Option<Card>> = use_signal(|| None);
    let mut original_copy_max: Signal<Option<CopyMax>> = use_signal(|| None);

    let mut load_error = use_signal(|| None::<String>);

    // commander search state
    let mut search_query = use_signal(|| String::new());
    let mut search_results = use_signal(|| Vec::<Card>::new());
    let mut is_searching = use_signal(|| false);
    let mut show_dropdown = use_signal(|| false);

    // defaults from resources
    use_effect(move || {
        let curr_deck = match &*deck_profile_resource.read() {
            Some(Ok(profile)) => profile.clone(),
            Some(Err(e)) => {
                load_error.set(Some(e.to_string()));
                return;
            }
            None => return,
        };

        deck_name.set(curr_deck.name.to_string());
        original_deck_name.set(curr_deck.name.to_string());
        copy_max.set(curr_deck.copy_max.clone());
        original_copy_max.set(curr_deck.copy_max.clone());

        let curr_commander = match &*commander_resource.read() {
            Some(Ok(Some(commander))) => commander.clone(),
            Some(Ok(None)) => return,
            Some(Err(e)) => {
                load_error.set(Some(e.to_string()));
                return;
            }
            None => return,
        };

        commander.set(Some(curr_commander.clone()));
        original_commander.set(Some(curr_commander.clone()));
        commander_display.set(curr_commander.scryfall_data.name.to_lowercase());
    });

    // save state
    let mut submission_error = use_signal(|| None::<String>);
    let mut is_saving = use_signal(|| false);

    // debounced search effect
    use_effect(move || {
        let query = search_query();

        if query.is_empty() {
            show_dropdown.set(false);
            return;
        }

        is_searching.set(true);

        spawn(async move {
            sleep(Duration::from_millis(500)).await;

            if let Some(sesh) = session() {
                let mut request = HttpSearchCards::by_name(&query);
                request.limit = Some(5);

                match client().search_cards(&request, &sesh).await {
                    Ok(cards) => {
                        search_results.set(cards);
                        is_searching.set(false);
                        show_dropdown.set(true);
                    }
                    Err(e) => {
                        tracing::error!("search error: {}", e);
                        is_searching.set(false);
                        show_dropdown.set(false);
                    }
                }
            }
        });
    });

    let mut attempt_submit = move || {
        submission_error.set(None);
        is_saving.set(true);

        spawn(async move {
            session.upkeep(client);
            let Some(sesh) = session() else {
                submission_error.set(Some("session expired".to_string()));
                is_saving.set(false);
                return;
            };

            let deck_name_update = if deck_name() != original_deck_name() {
                Some(deck_name())
            } else {
                None
            };

            let commander_id_update = if commander() != original_commander() {
                Some(commander().map(|c| c.card_profile.id))
            } else {
                None
            };

            let copy_max_update = if copy_max() != original_copy_max() {
                Some(copy_max().map(|cm| cm.max()))
            } else {
                None
            };

            let request = HttpUpdateDeckProfile::new(
                deck_name_update.as_deref(),
                commander_id_update,
                copy_max_update,
            );

            match client()
                .update_deck_profile(&deck_id, &request, &sesh)
                .await
            {
                Ok(created) => {
                    navigator.push(Router::ViewDeckProfile {
                        deck_id: created.id,
                    });
                }
                Err(e) => {
                    submission_error.set(Some(e.to_string()));
                    is_saving.set(false);
                }
            }
        });
    };

    let mut delete_error = use_signal(|| None::<String>);
    let mut attempt_delete = move || {
        session.upkeep(client);
        let Some(sesh) = session() else {
            submission_error.set(Some("session expired".to_string()));
            is_saving.set(false);
            return;
        };

        spawn(async move {
            match client().delete_deck(&deck_id, &sesh).await {
                Ok(_) => {
                    navigator.push(Router::DeckList {});
                }
                Err(e) => {
                    delete_error.set(Some(e.to_string()));
                }
            }
        });
    };

    rsx! {
        Bouncer {
            Swipeable { state: swipe_state, config: swipe_config,

                div { class : "form-container",
                    match &*deck_profile_resource.read() {
                        Some(Ok(profile)) => rsx! {

                            h2 { "{profile.name}" }

                            if let Some(error) = load_error() {
                                div { "{error}" }
                            }

                            form {
                                div { class : "form-group",
                                    input {
                                        id: "deck name",
                                        r#type : "text",
                                        placeholder : "deck name",
                                        value : "{deck_name}",
                                        autocapitalize : "none",
                                        spellcheck : "false",
                                        oninput : move |event| {
                                            deck_name.set(event.value());
                                        }
                                    }

                                    div { class: "commander-search",
                                        input {
                                            id: "commander",
                                            r#type : "text",
                                            placeholder : "commander",
                                            value : "{commander_display}",
                                            autocapitalize : "none",
                                            spellcheck : "false",
                                            oninput : move |event| {
                                                search_query.set(event.value());
                                                commander_display.set(event.value());
                                            }
                                        }

                                        if show_dropdown() {
                                            div { class: "dropdown",
                                                if is_searching() {
                                                    div { "searching..." }
                                                } else {
                                                    for card in search_results().iter().map(|x| x.clone()) {
                                                        div {
                                                            class: "dropdown-item",
                                                            onclick: move |_| {
                                                                commander.set(Some(card.clone()));
                                                                commander_display.set(card.scryfall_data.name.clone().to_lowercase());
                                                                show_dropdown.set(false);
                                                            },
                                                            {
                                                                format!("{}", card.scryfall_data.name.to_lowercase())
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }

                                    label { r#for : "copy-max", "card copy rule" }
                                    div {
                                        class: "form-group-copy-max",
                                        div {
                                            class: if copy_max() == Some(CopyMax::standard()) { "copy-max-box true" } else { "copy-max-box false" },
                                            onclick: move |_| {
                                                copy_max.set(Some(CopyMax::standard()));
                                            },
                                            "standard"
                                        }
                                        div {
                                            class: if copy_max() == Some(CopyMax::singleton()) { "copy-max-box true" } else { "copy-max-box false" },
                                            onclick: move |_| {
                                                copy_max.set(Some(CopyMax::singleton()));
                                            },
                                            "singleton"
                                        }
                                        div {
                                            class: if copy_max().is_none() { "copy-max-box true" } else { "copy-max-box false" },
                                            onclick: move |_| {
                                                copy_max.set(None);
                                            },
                                            "none"
                                        }
                                    }

                                    button {
                                        disabled: is_saving(),
                                        onclick : move |_| attempt_submit(),
                                        if is_saving() { "saving..." } else { "save" }
                                    }

                                    if let Some(error) = submission_error() {
                                        div { class: "error", "{error}" }
                                    }

                                    button {
                                        onclick : move |_| attempt_delete(),
                                        "delete"
                                    }

                                    button {
                                        onclick : move |_| {
                                            navigator.push(Router::DeckList {});
                                        },
                                        "back"
                                    }
                                }
                            }

                        },
                        Some(Err(e)) => rsx! { div { class : "error", "{e}"} },
                        None => rsx! { div { class : "spinning-card" } }
                    }
                }
            }
        }
    }
}
