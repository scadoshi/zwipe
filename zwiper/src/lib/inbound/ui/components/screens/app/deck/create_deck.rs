use dioxus::prelude::*;
use std::time::Duration;
use tokio::time::sleep;
use zwipe::{
    domain::{auth::models::session::Session, card::models::Card},
    inbound::http::handlers::card::search_card::HttpSearchCards,
};

use crate::{
    inbound::ui::{
        components::{
            auth::bouncer::Bouncer,
            interactions::swipe::{config::SwipeConfig, state::SwipeState, Swipeable},
        },
        router::Router,
    },
    outbound::client::{auth::AuthClient, card::search_cards::AuthClientSearchCards},
};

#[component]
pub fn CreateDeck() -> Element {
    let swipe_state = use_signal(|| SwipeState::new());
    let swipe_config = SwipeConfig::blank();

    let navigator = use_navigator();

    let session: Signal<Option<Session>> = use_context();
    let auth_client: Signal<AuthClient> = use_context();

    let mut deck_name = use_signal(|| String::new());
    let mut commander: Signal<Option<Card>> = use_signal(|| None);
    let mut commander_display = use_signal(|| String::new());
    let mut is_singleton = use_signal(|| true);

    // commander search state
    let mut search_query = use_signal(|| String::new());
    let mut search_results = use_signal(|| Vec::<Card>::new());
    let mut is_searching = use_signal(|| false);
    let mut show_dropdown = use_signal(|| false);

    // debounced search effect
    use_effect(move || {
        let query = search_query();

        if query.is_empty() {
            show_dropdown.set(false);
            return;
        }

        // do not show dropdown until search completes
        is_searching.set(true);

        // debounce: wait 300ms before searching
        spawn(async move {
            // wait 300ms before making the API call
            sleep(Duration::from_millis(300)).await;

            if let Some(sesh) = session() {
                let mut request = HttpSearchCards::by_name(&query);
                request.limit = Some(5);

                match auth_client().search_cards(&request, &sesh).await {
                    Ok(cards) => {
                        search_results.set(cards);
                        is_searching.set(false);
                        show_dropdown.set(true);
                    }
                    Err(e) => {
                        tracing::error!("search error: {}", e);
                        is_searching.set(false);
                    }
                }
            }
        });
    });

    rsx! {
        Bouncer {
                Swipeable { state: swipe_state, config: swipe_config,
                    div { class : "form-container",
                    form {
                        div { class : "form-group",
                            label { r#for : "deck-name", "" }
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

                            label { r#for : "commander", "" }
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
                                                        commander_display.set(card.scryfall_data.name.clone());
                                                        show_dropdown.set(false);
                                                    },
                                                    "{card.scryfall_data.name}"
                                                }
                                            }
                                        }
                                    }
                                }
                            }

                            label { r#for : "is-singleton", "card copy rule" }
                            div {
                                class: "form-group-singleton",
                                div {
                                    class: if is_singleton() { "singleton-box true" } else { "singleton-box false" },
                                    onclick: move |_| {
                                        is_singleton.set(true);
                                    },
                                    "singleton"
                                }
                                div {
                                    class: if !is_singleton() { "singleton-box true" } else { "singleton-box false" },
                                    onclick: move |_| {
                                        is_singleton.set(false);
                                    },
                                    "none"
                                }
                            }

                            button {
                                onclick : move |_| {
                                    tracing::info!("clicked save");
                                },
                                "save"
                            }

                            button {
                                onclick : move |_| {
                                    navigator.push(Router::DeckList {});
                                },
                                "back"
                            }
                        }
                    }
                }
            }
        }
    }
}
