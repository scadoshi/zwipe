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
            get_deck_profile::ClientGetDeckProfile, update_deck_profile::ClientUpdateDeckProfile,
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
        card::models::{search_card::SearchCards, Card},
        deck::models::deck::{
            copy_max::CopyMax, deck_profile::DeckProfile,
            update_deck_profile::InvalidUpdateDeckProfile,
        },
    },
    inbound::http::{
        handlers::deck::update_deck_profile::HttpUpdateDeckProfile, helpers::Optdate, ApiError,
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

    let deck_name_update = use_memo(move || {
        if deck_name() != original_deck_name() {
            Some(deck_name())
        } else {
            None
        }
    });

    let commander_id_update = use_memo(move || {
        if commander() != original_commander() {
            Optdate::Set(commander().map(|c| c.card_profile.id))
        } else {
            Optdate::Unchanged
        }
    });

    let copy_max_update = use_memo(move || {
        if copy_max() != original_copy_max() {
            Optdate::Set(copy_max().map(|cm| cm.max()))
        } else {
            Optdate::Unchanged
        }
    });

    let has_made_changes = use_memo(move || {
        deck_name_update().is_some()
            || commander_id_update().is_changed()
            || copy_max_update().is_changed()
    });

    let mut load_error = use_signal(|| None::<String>);

    // commander search state
    let mut search_query = use_signal(|| String::new());
    let mut search_results = use_signal(|| Vec::<Card>::new());
    let mut is_searching = use_signal(|| false);
    let mut show_dropdown = use_signal(|| false);

    // here bc need to pass to other needy components
    let filter = use_signal(|| SearchCards::default());
    let cards = use_signal(|| Vec::<Card>::new());

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
                let mut request = SearchCards::by_name(&query);
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

            if !has_made_changes() {
                submission_error.set(Some(InvalidUpdateDeckProfile::NoUpdates.to_string()));
                return;
            }

            let request = HttpUpdateDeckProfile::new(
                deck_name_update().as_deref(),
                commander_id_update(),
                copy_max_update(),
            );

            match client()
                .update_deck_profile(&deck_id, &request, &sesh)
                .await
            {
                Ok(_updated) => {
                    navigator.push(Router::ViewDeckProfile { deck_id });
                }
                Err(e) => {
                    submission_error.set(Some(e.to_string()));
                    is_saving.set(false);
                }
            }
        });
    };

    rsx! {
        Bouncer {
            Swipeable { state: swipe_state, config: swipe_config,

                div { class : "container-sm",
                    match &*deck_profile_resource.read() {
                        Some(Ok(_profile)) => rsx! {

                            h2 { class: "text-center mb-2 font-light tracking-wider", "{deck_name}" }

                            if let Some(error) = load_error() {
                                div { class: "message-error", "{error}" }
                            }

                            form { class: "flex-col text-center",
                                input { class: "input",
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

                                div { class: "mb-4",
                                    input { class: "input",
                                        id: "commander",
                                        r#type : "text",
                                        placeholder : "commander",
                                        value : "{commander_display}",
                                        autocapitalize : "none",
                                        spellcheck : "false",
                                        onclick : move |_| {
                                            search_query.set(String::new());
                                            commander_display.set(String::new());
                                            commander.set(None);
                                        },
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
                                                    div { class: "dropdown-item",
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

                                label { class: "label mb-2", r#for : "copy-max", "card copy rule" }
                                div { class: "flex gap-2 mb-2 flex-center",
                                    div { class: if copy_max() == Some(CopyMax::standard()) { "copy-max-box selected" } else { "copy-max-box unselected" },
                                        onclick: move |_| {
                                            copy_max.set(Some(CopyMax::standard()));
                                        },
                                        "standard"
                                    }
                                    div { class: if copy_max() == Some(CopyMax::singleton()) { "copy-max-box selected" } else { "copy-max-box unselected" },
                                        onclick: move |_| {
                                            copy_max.set(Some(CopyMax::singleton()));
                                        },
                                        "singleton"
                                    }
                                    div { class: if copy_max().is_none() { "copy-max-box selected" } else { "copy-max-box unselected" },
                                        onclick: move |_| {
                                            copy_max.set(None);
                                        },
                                        "none"
                                    }
                                }

                                if has_made_changes() {
                                    label { class : "label", "changes have been made" }
                                    button { class: "btn",
                                        disabled: is_saving(),
                                        onclick : move |_| attempt_submit(),
                                        if is_saving() { "saving..." } else { "save them" }
                                    }
                                }

                                if !has_made_changes() {
                                    label { class: "label mb-2", "deck cards" }
                                    div { class : "flex flex-between gap-2",
                                        button { class : "btn btn-half",
                                            onclick : move |_| {
                                                navigator.push(Router::AddDeckCard { deck_id, filter, cards});
                                            },
                                            "add"
                                        }

                                        button { class : "btn btn-half",
                                            onclick : move |_| {
                                                navigator.push(Router::RemoveDeckCard { deck_id, filter, cards});
                                            },
                                            "remove"
                                        }
                                    }
                                }

                                if let Some(error) = submission_error() {
                                    div { class: "message-error", "{error}" }
                                }

                                button { class: "btn",
                                    onclick : move |_| {
                                        navigator.push(Router::ViewDeckProfile { deck_id });
                                    },
                                    "back"
                                }
                            }

                        },
                        Some(Err(e)) => rsx! { div { class : "message-error", "{e}"} },
                        None => rsx! { div { class : "spinner" } }
                    }
                }
            }
        }
    }
}
