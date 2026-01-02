use crate::{
    inbound::{
        components::{
            auth::{bouncer::Bouncer, session_upkeep::Upkeep},
            fields::text_input::TextInput,
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
use std::time::Duration;
use tokio::time::sleep;
use uuid::Uuid;
use zwipe::{
    domain::{
        auth::models::session::Session,
        card::models::{
            search_card::card_filter::{builder::CardFilterBuilder, error::InvalidCardFilter},
            Card,
        },
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
pub fn EditDeck(deck_id: Uuid) -> Element {
    // config
    let navigator = use_navigator();
    let session: Signal<Option<Session>> = use_context();
    let client: Signal<ZwipeClient> = use_context();

    // original defaults and update intake
    let mut deck_name: Signal<String> = use_signal(String::new);
    let mut commander: Signal<Option<Card>> = use_signal(|| None);
    let mut commander_display = use_signal(String::new);
    let mut copy_max: Signal<Option<CopyMax>> = use_signal(|| None);

    // original
    let mut original_deck_name: Signal<String> = use_signal(String::new);
    let mut original_commander: Signal<Option<Card>> = use_signal(|| None);
    let mut original_copy_max: Signal<Option<CopyMax>> = use_signal(|| None);

    let mut load_error = use_signal(|| None::<String>);

    let original_deck_profile_resource: Resource<Result<DeckProfile, ApiError>> =
        use_resource(move || async move {
            session.upkeep(client);
            let Some(sesh) = session() else {
                return Err(ApiError::Unauthorized("session expired".to_string()));
            };
            client().get_deck_profile(deck_id, &sesh).await
        });
    use_effect(move || match original_deck_profile_resource() {
        Some(Ok(original)) => {
            original_deck_name.set(original.name.to_string());
            deck_name.set(original.name.to_string());
            original_copy_max.set(original.copy_max);
            copy_max.set(original.copy_max);
        }
        Some(Err(e)) => {
            load_error.set(Some(e.to_string()));
        }
        None => (),
    });
    let original_commander_resource: Resource<Result<Option<Card>, ApiError>> =
        use_resource(move || async move {
            let Some(Ok(DeckProfile {
                commander_id: Some(original_commander_id),
                ..
            })) = original_deck_profile_resource()
            else {
                return Ok(None);
            };
            session.upkeep(client);
            let Some(sesh) = session() else {
                return Err(ApiError::Unauthorized("session expired".to_string()));
            };
            client()
                .get_card(original_commander_id, &sesh)
                .await
                .map(Some)
        });
    use_effect(move || match original_commander_resource() {
        Some(Ok(Some(original))) => {
            original_commander.set(Some(original.clone()));
            commander.set(Some(original.clone()));
            commander_display.set(original.scryfall_data.name.to_lowercase());
        }
        Some(Ok(None)) | None => (),
        Some(Err(e)) => {
            load_error.set(Some(e.to_string()));
        }
    });

    let deck_name_update = use_memo(move || {
        if deck_name() != original_deck_name() {
            Some(deck_name())
        } else {
            None
        }
    });
    let commander_id_update = use_memo(move || {
        if commander() != original_commander() {
            Optdate::Set(commander().map(|c| c.scryfall_data.id))
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

    // commander search state
    let mut search_query = use_signal(String::new);
    let mut search_results = use_signal(Vec::<Card>::new);
    let mut is_searching = use_signal(|| false);
    let mut show_dropdown = use_signal(|| false);

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
                let Ok(card_filter) = CardFilterBuilder::with_name_contains(&query)
                    .set_limit(5)
                    .build()
                else {
                    tracing::error!("{}", InvalidCardFilter::Empty.to_string());
                    return;
                };
                match client().search_cards(&card_filter, &sesh).await {
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

            match client().update_deck_profile(deck_id, &request, &sesh).await {
                Ok(_updated) => {
                    submission_error.set(None);
                    is_saving.set(false);
                    navigator.push(Router::ViewDeck { deck_id });
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
            div { class: "page-header",
                h2 { "edit deck" }
            }

            div { class: "fixed top-0 left-0 h-screen flex flex-col items-center overflow-y-auto",
                style: "width: 100vw; justify-content: center; padding-top: 4rem;",
                div { class : "container-sm",
                    match &*original_deck_profile_resource.read() {
                        Some(Ok(_profile)) => rsx! {

                            if let Some(error) = load_error() {
                                // {tracing::error!("{}", error);}
                                div { class: "message-error", "{error}" }
                            }

                            form { class: "flex-col text-center",
                                TextInput {
                                    label: Some("deck name"),
                                    value: deck_name,
                                    id: "deck_name",
                                    placeholder: "deck name",
                                }

                                div { class: "mb-4",
                                    label { class: "label", "commander" }
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
                                                            card.scryfall_data.name.to_lowercase()
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }

                                label { class: "label mb-2", r#for : "copy-max", "card copy rule" }
                                div { class: "flex gap-2 mb-2 flex-center",
                                    div { class: if copy_max() == Some(CopyMax::standard()) { "type-box selected" } else { "type-box unselected" },
                                        onclick: move |_| {
                                            copy_max.set(Some(CopyMax::standard()));
                                        },
                                        "standard"
                                    }
                                    div { class: if copy_max() == Some(CopyMax::singleton()) { "type-box selected" } else { "type-box unselected" },
                                        onclick: move |_| {
                                            copy_max.set(Some(CopyMax::singleton()));
                                        },
                                        "singleton"
                                    }
                                    div { class: if copy_max().is_none() { "type-box selected" } else { "type-box unselected" },
                                        onclick: move |_| {
                                            copy_max.set(None);
                                        },
                                        "none"
                                    }
                                }

                                if let Some(error) = submission_error() {
                                    div { class: "message-error", "{error}" }
                                }
                            }

                        },
                        Some(Err(e)) => rsx! { div { class : "message-error", "{e}"} },
                        None => rsx! { div { class : "spinner" } }
                    }
                }
            }

            div { class: "util-bar",
                button {
                    class: "util-btn",
                    onclick: move |_| {
                        navigator.push(Router::ViewDeck { deck_id });
                    },
                    "back"
                }
                if !has_made_changes() {
                    button {
                        class : "util-btn",
                        onclick : move |_| {
                            navigator.push(Router::AddDeckCard { deck_id });
                        },
                        "add cards"
                    }
                    button {
                        class : "util-btn",
                        onclick : move |_| {
                            navigator.push(Router::RemoveDeckCard { deck_id });
                        },
                        "remove cards"
                    }
                }
                if has_made_changes() {
                    button {
                        class: "util-btn",
                        disabled: is_saving(),
                        onclick : move |_| attempt_submit(),
                            if is_saving() { "saving..." } else { "save changes" }
                    }
                }
            }
        }
    }
}
