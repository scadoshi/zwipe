use crate::{
    inbound::ui::{
        components::{
            auth::bouncer::Bouncer,
            interactions::swipe::{config::SwipeConfig, state::SwipeState, Swipeable},
        },
        router::Router,
    },
    outbound::client::{
        auth::{session::AuthClientSession, AuthClient},
        deck::get_deck_profiles::{GetDeckProfilesError, GetDecks},
    },
};
use dioxus::prelude::*;
use zwipe::domain::{
    auth::models::session::Session, deck::models::deck::deck_profile::DeckProfile,
};

#[component]
pub fn Decks() -> Element {
    let swipe_state = use_signal(|| SwipeState::new());
    let swipe_config = SwipeConfig::blank();

    let navigator = use_navigator();
    let auth_client: Signal<AuthClient> = use_context();
    let mut session: Signal<Option<Session>> = use_context();

    let deck_profiles_resource: Resource<Result<Vec<DeckProfile>, GetDeckProfilesError>> =
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
        Bouncer {
            Swipeable { state: swipe_state, config: swipe_config,
                div { class : "decks-container",
                    h2 { "decks" }

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
                                    div { class : "deck-profiles",
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
                            div { class : "error",
                                p { "{e}" }
                            }
                        },
                        None => rsx! {
                            div { class: "spinning-card" }
                        },

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
