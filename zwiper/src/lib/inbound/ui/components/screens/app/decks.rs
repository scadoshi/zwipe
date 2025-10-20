use crate::{
    inbound::ui::router::Router,
    outbound::client::{
        auth::{session::ActiveSession, AuthClient},
        deck::get_deck_profiles::{GetDeckProfilesError, GetDecks},
    },
};
use dioxus::prelude::*;
use uuid::Uuid;
use zwipe::domain::{
    auth::models::session::Session,
    deck::models::deck::{deck_name::DeckName, deck_profile::DeckProfile},
};

#[component]
pub fn Decks() -> Element {
    let navigator = use_navigator();
    let auth_client: Signal<AuthClient> = use_context();
    let mut session: Signal<Option<Session>> = use_context();

    let decks: Resource<Result<Vec<DeckProfile>, GetDeckProfilesError>> =
        use_resource(move || async move {
            let Some(current) = session.read().clone() else {
                return Err(GetDeckProfilesError::SessionExpired);
            };

            let Some(active) = auth_client
                .read()
                .infallible_get_active_session(&current)
                .await
            else {
                return Err(GetDeckProfilesError::SessionExpired);
            };

            let result = auth_client.read().get_deck_profiles(&active).await;

            if active != current {
                session.set(Some(active));
            }

            result
        });

    rsx! {
        div { class : "nicely-centered",
            div { class : "decks-container",
                h2 { "decks" }

                {
                    let is_empty = decks.value().with(|val| {
                        matches!(val, Some(Ok(list)) if list.is_empty())
                    });
                    let has_data = decks.value().with(|val| {
                        matches!(val, Some(Ok(list)) if !list.is_empty())
                    });
                    let has_error = decks.value().with(|val| matches!(val, Some(Err(_))));

                    if is_empty {
                        rsx! {
                            div { class: "empty-message",
                                p { "no decks yet" }
                            }
                            button {
                                onclick : move |_| {
                                    navigator.push(Router::Home {});
                                },
                                "back"
                            }
                        }
                    } else if has_data {
                        let deck_profiles = decks.value().with(|val| {
                            if let Some(Ok(deck_list)) = val {
                                deck_list.iter().map(|d| (d.id, d.name.clone())).collect::<Vec<_>>()
                            } else {
                                vec![]
                            }
                        });

                        rsx! {
                            div { class: "deck-list",
                                for (id, name) in deck_profiles {
                                    div {
                                        key: "{id}",
                                        class : "deck-item",
                                        onclick : move |_| {
                                            tracing::info!("clicked deck: {name}");
                                        },
                                        h3 { { name.to_string() } }
                                    }
                                }
                            }
                            button {
                                onclick : move |_| {
                                    navigator.push(Router::Home {});
                                },
                                "back"
                            }
                        }
                    } else if has_error {
                        rsx! {
                            div { class : "error",
                                p { "failed to load decks" }
                            }
                            button {
                                onclick : move |_| {
                                    navigator.push(Router::Home {});
                                },
                                "back"
                            }
                        }
                    } else {
                        rsx! {
                            div { class: "spinning-card" }
                        }
                    }
                }
            }
        }
    }
}
