use crate::{
    inbound::{
        components::auth::{bouncer::Bouncer, session_upkeep::Upkeep},
        router::Router,
    },
    outbound::client::{deck::get_deck_profiles::ClientGetDeckList, ZwipeClient},
};
use dioxus::prelude::*;
use zwipe::{
    domain::{auth::models::session::Session, deck::models::deck::deck_profile::DeckProfile},
    inbound::http::ApiError,
};

#[component]
pub fn DeckList() -> Element {
    let navigator = use_navigator();
    let auth_client: Signal<ZwipeClient> = use_context();
    let session: Signal<Option<Session>> = use_context();

    let mut deck_profiles_resource: Resource<Result<Vec<DeckProfile>, ApiError>> =
        use_resource(move || async move {
            session.upkeep(auth_client);
            let Some(session) = session() else {
                return Err(ApiError::Unauthorized("session expired".to_string()));
            };

            auth_client().get_deck_profiles(&session).await
        });

    // Restart resource on component mount to ensure fresh data
    use_effect(move || {
        deck_profiles_resource.restart();
    });

    rsx! {
        Bouncer {
            div { class: "page-header",
                h2 { "decks" }
            }

            div { class: "left-0 h-screen flex flex-col items-center overflow-y-auto",
                style: "width: 100vw; padding-top: 4rem; padding-bottom: 5rem;",
                div { class: "flex-col",
                    style: "max-width: 40rem; width: 100%; padding: 2rem;",

                    match &*deck_profiles_resource.read() {
                        Some(Ok(deck_profiles)) => {
                            if deck_profiles.is_empty() {
                                rsx! {
                                    div { class: "message-empty",
                                        p { "no decks yet" }
                                    }
                                }
                            } else {
                                rsx! {
                                    for profile in deck_profiles.iter().map(|x| x.to_owned()) {
                                        div { class : "card",
                                            key : "{profile.id}",
                                            onclick : move |_| {
                                                navigator.push(Router::ViewDeck {
                                                    deck_id: profile.id,
                                                });
                                            },
                                            h3 { class: "font-light text-base tracking-wide",
                                                { profile.name.to_string() }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        Some(Err(e)) => rsx!{
                            div { class : "message-empty",
                                p { class : "message-error", "{e}" }
                            }
                        },
                        None => rsx! {
                            div { class: "message-empty",
                                div { class: "spinner" }
                            }
                        },
                    }

                }
            }

            div { class: "util-bar",
                button {
                    class: "util-btn",
                    onclick: move |_| {
                        navigator.push(Router::Home {});
                    },
                    "back"
                }
                button {
                    class: "util-btn",
                    onclick: move |_| {
                        navigator.push(Router::CreateDeck);
                    },
                    "create deck"
                }
            }
        }
    }
}
