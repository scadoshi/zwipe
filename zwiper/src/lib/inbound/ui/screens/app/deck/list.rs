use crate::{
    inbound::ui::{
        components::{
            auth::{bouncer::Bouncer, session_upkeep::Upkeep},
            interactions::swipe::{config::SwipeConfig, state::SwipeState, Swipeable},
        },
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
    let swipe_state = use_signal(SwipeState::new);
    let swipe_config = SwipeConfig::blank();

    let navigator = use_navigator();
    let auth_client: Signal<ZwipeClient> = use_context();
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
                div { class : "container-sm flex-col",
                    h2 { class: "text-center mb-4 font-light tracking-wider", "deck list" }

                    div { class : "flex-col min-h-50 p-4 border rounded mb-4",
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
                                        div { class : "flex-1 overflow-y-auto",
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

                    div { class: "flex-col",
                        button { class: "btn",
                            onclick : move |_| {
                                navigator.push(Router::CreateDeck);
                            },
                            "create deck"
                        }
                        button { class: "btn",
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
