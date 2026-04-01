//! Edit deck screen.

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
            get_deck::ClientGetDeck, update_deck_profile::ClientUpdateDeckProfile,
        },
        ZwipeClient,
    },
};
use dioxus::prelude::*;
use dioxus_primitives::toast::{use_toast, ToastOptions};
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
            Deck, deck_profile::DeckProfile, format::Format,
            update_deck_profile::InvalidUpdateDeckProfile,
        },
    },
    inbound::http::{
        handlers::deck::update_deck_profile::HttpUpdateDeckProfile, helpers::Optdate, ApiError,
    },
};

/// Screen for editing a deck with name and settings.
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
    let mut selected_format: Signal<Option<Format>> = use_signal(|| None);

    // original
    let mut original_deck_name: Signal<String> = use_signal(String::new);
    let mut original_commander: Signal<Option<Card>> = use_signal(|| None);
    let mut original_format: Signal<Option<Format>> = use_signal(|| None);

    let toast = use_toast();

    let original_deck_resource: Resource<Result<Deck, ApiError>> =
        use_resource(move || async move {
            session.upkeep(client);
            let Some(session) = session() else {
                return Err(ApiError::Unauthorized("session expired".to_string()));
            };
            client().get_deck(deck_id, &session).await
        });
    use_effect(move || match original_deck_resource() {
        Some(Ok(deck)) => {
            original_deck_name.set(deck.deck_profile.name.to_string());
            deck_name.set(deck.deck_profile.name.to_string());
            original_format.set(deck.deck_profile.format);
            selected_format.set(deck.deck_profile.format);
        }
        Some(Err(e)) => {
            toast.error(e.to_string(), ToastOptions::default().duration(Duration::from_millis(3000)));
        }
        None => (),
    });
    let original_commander_resource: Resource<Result<Option<Card>, ApiError>> =
        use_resource(move || async move {
            let Some(Ok(Deck {
                deck_profile:
                    DeckProfile {
                        commander_id: Some(original_commander_id),
                        ..
                    },
                ..
            })) = original_deck_resource()
            else {
                return Ok(None);
            };
            session.upkeep(client);
            let Some(session) = session() else {
                return Err(ApiError::Unauthorized("session expired".to_string()));
            };
            client()
                .get_card(original_commander_id, &session)
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
            toast.error(e.to_string(), ToastOptions::default().duration(Duration::from_millis(3000)));
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
    let format_update = use_memo(move || {
        if selected_format() != original_format() {
            Optdate::Set(selected_format().map(|f| f.to_legality_key().to_string()))
        } else {
            Optdate::Unchanged
        }
    });
    let has_made_changes = use_memo(move || {
        deck_name_update().is_some()
            || commander_id_update().is_changed()
            || format_update().is_changed()
    });

    // commander enabled only when format has_commander
    let commander_enabled = use_memo(move || {
        selected_format().is_some_and(|f| f.has_commander())
    });

    // commander search state
    let mut search_query = use_signal(String::new);
    let mut search_results = use_signal(Vec::<Card>::new);
    let mut is_searching = use_signal(|| false);
    let mut show_dropdown = use_signal(|| false);

    // save state
    let mut is_saving = use_signal(|| false);

    // debounced commander search effect
    use_effect(move || {
        let query = search_query();

        if query.len() < 3 {
            show_dropdown.set(false);
            is_searching.set(false);
            return;
        }

        is_searching.set(true);

        spawn(async move {
            sleep(Duration::from_millis(800)).await;

            // only proceed if the query hasn't changed during the delay
            if search_query() != query {
                return;
            }

            if let Some(session) = session() {
                let Ok(card_filter) = CardFilterBuilder::with_name_contains(&query)
                    .set_limit(5)
                    .build()
                else {
                    tracing::error!("{}", InvalidCardFilter::Empty.to_string());
                    return;
                };
                match client().search_cards(&card_filter, &session).await {
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

    let mut do_submit = move || {
        is_saving.set(true);

        spawn(async move {
            session.upkeep(client);
            let Some(session) = session() else {
                toast.error("session expired".to_string(), ToastOptions::default().duration(Duration::from_millis(3000)));
                is_saving.set(false);
                return;
            };

            if !has_made_changes() {
                toast.error(InvalidUpdateDeckProfile::NoUpdates.to_string(), ToastOptions::default().duration(Duration::from_millis(3000)));
                is_saving.set(false);
                return;
            }

            let request = HttpUpdateDeckProfile::new(
                deck_name_update().as_deref(),
                commander_id_update(),
                format_update(),
            );

            match client()
                .update_deck_profile(deck_id, &request, &session)
                .await
            {
                Ok(_updated) => {
                    is_saving.set(false);
                    navigator.push(Router::ViewDeck { deck_id });
                }
                Err(e) => {
                    toast.error(e.to_string(), ToastOptions::default().duration(Duration::from_millis(3000)));
                    is_saving.set(false);
                }
            }
        });
    };

    let mut attempt_submit = move || {
        do_submit();
    };

    rsx! {
        Bouncer {
            div { class: "screen",
                div { class: "page-header",
                    h2 { "edit deck" }
                }

                div { class: "screen-content centered content-enter",
                div { class : "container-sm",
                    match &*original_deck_resource.read() {
                        Some(Ok(_deck)) => rsx! {
                            form { class: "flex-col text-center",
                                TextInput {
                                    label: "deck name",
                                    value: deck_name,
                                    id: "deck_name",
                                    placeholder: "deck name",
                                }

                                // format selector (chips)
                                div { class: "mb-4",
                                    div { class: "label-row",
                                        label { class: "label", "format" }
                                        if selected_format().is_some() {
                                            button {
                                                class: "clear-btn",
                                                onclick: move |_| {
                                                    selected_format.set(None);
                                                    commander.set(None);
                                                    commander_display.set(String::new());
                                                },
                                                "\u{00d7}"
                                            }
                                        }
                                    }
                                    div { class: "flex flex-wrap gap-1 flex-center",
                                        for fmt in Format::all().iter().copied() {
                                            div {
                                                class: if selected_format() == Some(fmt) { "chip selected" } else { "chip" },
                                                onclick: move |_| {
                                                    if selected_format() == Some(fmt) {
                                                        selected_format.set(None);
                                                        commander.set(None);
                                                        commander_display.set(String::new());
                                                    } else {
                                                        selected_format.set(Some(fmt));
                                                        if !fmt.has_commander() {
                                                            commander.set(None);
                                                            commander_display.set(String::new());
                                                        }
                                                    }
                                                },
                                                { fmt.display_name().to_lowercase() }
                                            }
                                        }
                                    }
                                }

                                // commander selector (greyed out unless format has_commander)
                                div { class: "mb-4",
                                    div { class: "label-row",
                                        label {
                                            class: if commander_enabled() { "label" } else { "label text-muted" },
                                            "commander"
                                        }
                                        if commander().is_some() && commander_enabled() {
                                            button {
                                                class: "clear-btn",
                                                onclick: move |_| {
                                                    commander.set(None);
                                                    commander_display.set(String::new());
                                                    search_query.set(String::new());
                                                    show_dropdown.set(false);
                                                },
                                                "x"
                                            }
                                        }
                                    }
                                    input { class: if commander_enabled() { "input" } else { "input disabled" },
                                        id: "commander",
                                        r#type : "text",
                                        placeholder: if commander_enabled() { "commander" } else { "select a commander format" },
                                        value : "{commander_display}",
                                        autocapitalize : "none",
                                        spellcheck : "false",
                                        disabled: !commander_enabled(),
                                        onclick : move |_| {
                                            if commander_enabled() {
                                                search_query.set(String::new());
                                                commander_display.set(String::new());
                                                commander.set(None);
                                            }
                                        },
                                        oninput : move |event| {
                                            if commander_enabled() {
                                                search_query.set(event.value());
                                                commander_display.set(event.value());
                                            }
                                        }
                                    }

                                    if show_dropdown() && commander_enabled() {
                                        div { class: "dropdown",
                                            if is_searching() {
                                                div { "searching..." }
                                            } else if search_results().is_empty() {
                                                div { "no results" }
                                            } else {
                                                for card in search_results().iter().cloned() {
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

                            }

                        },
                        Some(Err(_)) => rsx! { p { class: "text-muted", "could not load deck" } },
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
}
