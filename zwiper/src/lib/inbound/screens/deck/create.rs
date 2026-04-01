//! Create new deck screen.

use crate::{
    inbound::{
        components::{
            auth::{bouncer::Bouncer, session_upkeep::Upkeep},
            fields::text_input::TextInput,
        },
        router::Router,
    },
    outbound::client::{
        card::search_cards::ClientSearchCards, deck::create_deck::ClientCreateDeck, ZwipeClient,
    },
};
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use std::time::Duration;
use tokio::time::sleep;
use zwipe::{
    domain::{
        auth::models::session::Session,
        card::models::{
            search_card::card_filter::{builder::CardFilterBuilder, error::InvalidCardFilter},
            Card,
        },
        deck::models::deck::format::Format,
    },
    inbound::http::handlers::deck::create_deck_profile::HttpCreateDeckProfile,
};

/// Screen for creating a new deck with name and settings.
#[component]
pub fn CreateDeck() -> Element {
    let navigator = use_navigator();

    let session: Signal<Option<Session>> = use_context();
    let auth_client: Signal<ZwipeClient> = use_context();

    // form
    let deck_name = use_signal(String::new);
    let mut selected_format: Signal<Option<Format>> = use_signal(|| None);
    let mut format_display = use_signal(String::new);
    let mut format_show_dropdown = use_signal(|| false);
    let mut commander: Signal<Option<Card>> = use_signal(|| None);
    let mut commander_display = use_signal(String::new);
    // commander search state
    let mut search_query = use_signal(String::new);
    let mut search_results = use_signal(Vec::<Card>::new);
    let mut is_searching = use_signal(|| false);
    let mut show_dropdown = use_signal(|| false);

    // save state
    let toast = use_toast();
    let mut is_saving = use_signal(|| false);

    // commander enabled only when format has_commander
    let commander_enabled = use_memo(move || {
        selected_format().is_some_and(|f| f.has_commander())
    });

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
                    .set_is_commander(true)
                    .set_limit(5)
                    .build()
                else {
                    tracing::error!("{}", InvalidCardFilter::Empty.to_string());
                    return;
                };

                match auth_client().search_cards(&card_filter, &session).await {
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
        is_saving.set(true);

        spawn(async move {
            session.upkeep(auth_client);
            let Some(session) = session() else {
                toast.error("session expired".to_string(), ToastOptions::default().duration(Duration::from_millis(3000)));
                is_saving.set(false);
                return;
            };

            let commander_id = commander().map(|c| c.scryfall_data.id);
            let format_str = selected_format().map(|f| f.to_legality_key().to_string());
            let request =
                HttpCreateDeckProfile::new(&deck_name(), commander_id, format_str);

            match auth_client().create_deck_profile(&request, &session).await {
                Ok(created) => {
                    navigator.push(Router::ViewDeck {
                        deck_id: created.id,
                    });
                }
                Err(e) => {
                    toast.error(e.to_string().to_lowercase(), ToastOptions::default().duration(Duration::from_millis(3000)));
                    is_saving.set(false);
                }
            }
        });
    };

    // filtered format list based on typed input
    let format_options: Vec<Format> = {
        let query = format_display().to_lowercase();
        if query.is_empty() {
            Format::all().to_vec()
        } else {
            Format::all()
                .iter()
                .filter(|f| f.display_name().to_lowercase().contains(&query))
                .copied()
                .collect()
        }
    };

    rsx! {
        Bouncer {
            div { class: "screen",
                div { class: "page-header",
                    h2 { "create deck" }
                }

                div { class: "screen-content centered content-enter",
                div { class : "container-sm",

                    form { class : "flex-col text-center",
                        TextInput {
                            label: "deck name",
                            value: deck_name,
                            id: "deck_name",
                            placeholder: "deck name",
                        }

                        // format selector
                        div { class: "mb-4",
                            label { class: "label", r#for: "format", "format" }
                            input { class: "input",
                                id: "format",
                                r#type: "text",
                                placeholder: "format",
                                value: "{format_display}",
                                autocapitalize: "none",
                                spellcheck: "false",
                                onclick: move |_| {
                                    format_display.set(String::new());
                                    selected_format.set(None);
                                    commander.set(None);
                                    commander_display.set(String::new());
                                    format_show_dropdown.set(true);
                                },
                                oninput: move |event| {
                                    format_display.set(event.value());
                                    selected_format.set(None);
                                    format_show_dropdown.set(true);
                                }
                            }

                            if format_show_dropdown() {
                                div { class: "dropdown",
                                    if format_options.is_empty() {
                                        div { "no results" }
                                    } else {
                                        for fmt in format_options.iter().copied() {
                                            div { class: "dropdown-item",
                                                onclick: move |_| {
                                                    selected_format.set(Some(fmt));
                                                    format_display.set(fmt.display_name().to_lowercase());
                                                    format_show_dropdown.set(false);
                                                    // clear commander if format doesn't support it
                                                    if !fmt.has_commander() {
                                                        commander.set(None);
                                                        commander_display.set(String::new());
                                                    }
                                                },
                                                { fmt.display_name().to_lowercase() }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        // commander selector (greyed out unless format has_commander)
                        div { class: "mb-4",
                            label {
                                class: if commander_enabled() { "label" } else { "label text-muted" },
                                r#for: "commander",
                                "commander"
                            }
                            input { class: if commander_enabled() { "input" } else { "input disabled" },
                                id: "commander",
                                r#type: "text",
                                placeholder: if commander_enabled() { "commander" } else { "select a commander format" },
                                value: "{commander_display}",
                                autocapitalize: "none",
                                spellcheck: "false",
                                disabled: !commander_enabled(),
                                onclick: move |_| {
                                    if commander_enabled() {
                                        search_query.set(String::new());
                                        commander_display.set(String::new());
                                    }
                                },
                                oninput: move |event| {
                                    if commander_enabled() {
                                        search_query.set(event.value());
                                        commander_display.set(event.value());
                                    }
                                }
                            }

                            if show_dropdown() && commander_enabled() {
                                div { class : "dropdown",
                                    if is_searching() {
                                        div { "searching..." }
                                    } else {
                                        if search_results().is_empty() {
                                            div { "no results" }
                                        } else {
                                            for card in search_results() {
                                                div { class : "dropdown-item",
                                                    onclick : move |_| {
                                                        commander.set(Some(card.clone()));
                                                        commander_display.set(card.scryfall_data.name.to_lowercase());
                                                        show_dropdown.set(false);
                                                    },
                                                    { card.scryfall_data.name.to_lowercase() }
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
                button { class : "util-btn",
                    disabled: is_saving(),
                    onclick : move |_| attempt_submit(),
                    if is_saving() { "saving..." } else { "save" }
                }
            }
            }
        }
    }
}
