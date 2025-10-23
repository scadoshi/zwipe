use crate::{
    inbound::ui::{
        components::{
            auth::{bouncer::Bouncer, session_upkeep::Upkeep},
            interactions::swipe::{config::SwipeConfig, state::SwipeState, Swipeable},
        },
        router::Router,
    },
    outbound::client::{auth::AuthClient, deck::get_deck_profiles::AuthClientGetDecks},
};
use dioxus::prelude::*;
use zwipe::{
    domain::{auth::models::session::Session, deck::models::deck::deck_profile::DeckProfile},
    inbound::http::ApiError,
};

#[component]
pub fn DeckList() -> Element {
    let swipe_state = use_signal(|| SwipeState::new());
    let swipe_config = SwipeConfig::blank();

    let navigator = use_navigator();
    let auth_client: Signal<AuthClient> = use_context();
    let session: Signal<Option<Session>> = use_context();

    let deck_profiles_resource: Resource<Result<Vec<DeckProfile>, ApiError>> =
        use_resource(move || async move {
            session.upkeep(auth_client);
            let Some(sesh) = session() else {
                return Err(ApiError::Unauthorized("session expired".to_string()));
            };

            auth_client().get_deck_profiles(&sesh).await
        });

    rsx! {
        Bouncer {
            Swipeable { state: swipe_state, config: swipe_config,
                div { class : "decks-wrapper",
                    h2 { "decks" }

                    div { class : "decks-container",
                        match &*deck_profiles_resource.read() {
                            Some(Ok(deck_profiles)) => {
                                if deck_profiles.is_empty() {
                                    rsx! {
                                        div { class: "empty-message",
                                            p { "no decks yet" }
                                        }
                                    }
                                } else {
                                    rsx! {
                                        div { class : "deck-list",
                                            for profile in deck_profiles.iter().map(|x| x.to_owned()) {
                                                div {
                                                    key : "{profile.id}",
                                                    class : "deck-item",
                                                    onclick : move |_| {
                                                        tracing::info!("clicked into deck {}", profile.name.to_string());
                                                    },
                                                    h3 { { profile.name.to_string() } }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            Some(Err(e)) => rsx!{
                                div { class : "empty-message",
                                    p { class : "error", "{e}" }
                                }
                            },
                            None => rsx! {
                                div { class: "empty-message",
                                    div { class: "spinning-card" }
                                }
                            },
                        }
                    }

                    div { class: "deck-buttons",
                        button {
                            onclick : move |_| {
                                navigator.push(Router::CreateDeck);
                            },
                            "add deck"
                        }
                        button {
                            onclick : move |_| {
                                navigator.push(Router::Home {});
                            },
                            "back"
                        }
                    }
                }
            }
        }
    }
}
