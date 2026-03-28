//! Edit deck screen.

use crate::inbound::components::alert_dialog::{
    AlertDialogAction, AlertDialogActions, AlertDialogCancel, AlertDialogContent,
    AlertDialogDescription, AlertDialogRoot, AlertDialogTitle,
};
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
            Deck, copy_max::CopyMax, deck_profile::DeckProfile,
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
    let mut copy_max: Signal<Option<CopyMax>> = use_signal(|| None);

    // original
    let mut original_deck_name: Signal<String> = use_signal(String::new);
    let mut original_commander: Signal<Option<Card>> = use_signal(|| None);
    let mut original_copy_max: Signal<Option<CopyMax>> = use_signal(|| None);
    let mut max_entry_quantity: Signal<i32> = use_signal(|| 0);

    let mut load_error = use_signal(|| None::<String>);

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
            original_copy_max.set(deck.deck_profile.copy_max);
            copy_max.set(deck.deck_profile.copy_max);
            max_entry_quantity.set(
                deck.entries
                    .iter()
                    .filter(|e| {
                        !e.card.scryfall_data.is_basic_land()
                    })
                    .map(|e| *e.deck_card.quantity)
                    .max()
                    .unwrap_or(0),
            );
        }
        Some(Err(e)) => {
            load_error.set(Some(e.to_string()));
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
            Optdate::Set(copy_max().map(|cm| *cm))
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
    let mut show_truncation_warning = use_signal(|| false);

    let would_truncate = use_memo(move || {
        let Some(new_max) = copy_max() else {
            return false;
        };
        max_entry_quantity() > *new_max
    });

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
            if let Some(session) = session() {
                let Ok(card_filter) = CardFilterBuilder::with_name_contains(&query)
                    .set_is_valid_commander(true)
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
        submission_error.set(None);
        is_saving.set(true);

        spawn(async move {
            session.upkeep(client);
            let Some(session) = session() else {
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
                .update_deck_profile(deck_id, &request, &session)
                .await
            {
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

    let mut attempt_submit = move || {
        if would_truncate() {
            show_truncation_warning.set(true);
            return;
        }
        do_submit();
    };

    rsx! {
        Bouncer {
            div { class: "screen",
                div { class: "page-header",
                    h2 { "edit deck" }
                }

                div { class: "screen-content centered",
                div { class : "container-sm",
                    match &*original_deck_resource.read() {
                        Some(Ok(_deck)) => rsx! {
                            if let Some(error) = load_error() {
                                div { class: "message-error", "{error}" }
                            }

                            form { class: "flex-col text-center",
                                TextInput {
                                    label: "deck name",
                                    value: deck_name,
                                    id: "deck_name",
                                    placeholder: "deck name",
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

                                div { class: "mb-4",
                                    div { class: "label-row",
                                        label { class: "label", "commander" }
                                        if commander().is_some() {
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
                if has_made_changes() {
                    button {
                        class: "util-btn",
                        disabled: is_saving(),
                        onclick : move |_| attempt_submit(),
                            if is_saving() { "saving..." } else { "save changes" }
                    }
                }
            }

            AlertDialogRoot {
                open: show_truncation_warning(),
                on_open_change: move |open| show_truncation_warning.set(open),
                AlertDialogContent {
                    AlertDialogTitle { "copy rule warning" }
                    AlertDialogDescription {
                        "some cards in this deck exceed the new copy limit. "
                        "their quantities will be truncated down to the new maximum. "
                        "this cannot be undone."
                    }
                    AlertDialogActions {
                        AlertDialogCancel {
                            on_click: move |_| show_truncation_warning.set(false),
                            "cancel"
                        }
                        AlertDialogAction {
                            on_click: move |_| {
                                show_truncation_warning.set(false);
                                do_submit();
                            },
                            "confirm"
                        }
                    }
                }
            }
            }
        }
    }
}
