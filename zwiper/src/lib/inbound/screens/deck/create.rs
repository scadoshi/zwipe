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
use std::time::Duration;
use tokio::time::sleep;
use zwipe::{
    domain::{
        auth::models::session::Session,
        card::models::{
            search_card::card_filter::{builder::CardFilterBuilder, error::InvalidCardFilter},
            Card,
        },
        deck::models::deck::copy_max::CopyMax,
    },
    inbound::http::handlers::deck::create_deck_profile::HttpCreateDeckProfile,
};

#[component]
pub fn CreateDeck() -> Element {
    let navigator = use_navigator();

    let session: Signal<Option<Session>> = use_context();
    let auth_client: Signal<ZwipeClient> = use_context();

    // form
    let deck_name = use_signal(String::new);
    let mut commander: Signal<Option<Card>> = use_signal(|| None);
    let mut commander_display = use_signal(String::new);
    let mut copy_max: Signal<Option<CopyMax>> = use_signal(|| None);

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

                match auth_client().search_cards(&card_filter, &sesh).await {
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
            session.upkeep(auth_client);
            let Some(sesh) = session() else {
                submission_error.set(Some("session expired".to_string()));
                is_saving.set(false);
                return;
            };

            let commander_id = commander().map(|c| c.scryfall_data.id);
            let request =
                HttpCreateDeckProfile::new(&deck_name(), commander_id, copy_max().map(|x| x.max()));

            match auth_client().create_deck_profile(&request, &sesh).await {
                Ok(created) => {
                    navigator.push(Router::ViewDeck {
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

    rsx! {
        Bouncer {
            div { class: "page-header",
                h2 { "create deck" }
            }

            div { class: "sticky top-0 left-0 h-screen flex flex-col items-center overflow-y-auto",
                style: "width: 100vw; justify-content: center; padding-top: 4rem;",
                div { class : "container-sm",

                    form { class : "flex-col text-center",
                        TextInput {
                            label: "deck name",
                            value: deck_name,
                            id: "deck_name",
                            placeholder: "deck name",
                        }

                        div { class : "mb-4",
                            label { class : "label", r#for : "commander", "commander" }
                            input { class : "input",
                                id : "commander",
                                r#type : "text",
                                placeholder : "commander",
                                value : "{commander_display}",
                                autocapitalize : "none",
                                spellcheck : "false",
                                onclick : move |_| {
                                    search_query.set(String::new());
                                    commander_display.set(String::new());
                                },
                                oninput : move |event| {
                                    search_query.set(event.value());
                                    commander_display.set(event.value());
                                }
                            }

                            if show_dropdown() {
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

                        label { class : "label", r#for : "copy-max", "card copy rule" }
                        div { class : "flex gap-2 mb-4 flex-center",
                            div { class : if copy_max() == Some(CopyMax::standard()) { "type-box selected" } else { "type-box unselected" },
                                onclick: move |_| {
                                    copy_max.set(Some(CopyMax::standard()));
                                },
                                "standard"
                            }
                            div { class : if copy_max() == Some(CopyMax::singleton()) { "type-box selected" } else { "type-box unselected" },
                                onclick: move |_| {
                                    copy_max.set(Some(CopyMax::singleton()));
                                },
                                "singleton"
                            }
                            div { class : if copy_max().is_none() { "type-box selected" } else { "type-box unselected" },
                                onclick: move |_| {
                                    copy_max.set(None);
                                },
                                "none"
                            }
                        }

                        if let Some(error) = submission_error() {
                            div { class : "message-error", "{error}" }
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
                    if is_saving() { "saving..." } else { "save deck" }
                }
            }
        }
    }
}
