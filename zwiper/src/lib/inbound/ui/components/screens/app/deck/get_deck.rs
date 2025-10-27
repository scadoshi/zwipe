use crate::{
    inbound::ui::{
        components::{
            auth::{bouncer::Bouncer, session_upkeep::Upkeep},
            interactions::swipe::{config::SwipeConfig, state::SwipeState, Swipeable},
        },
        router::Router,
    },
    outbound::client::{
        auth::AuthClient, card::get_card::AuthClientGetCard,
        deck::get_deck_profile::AuthClientGetDeckProfile,
    },
};
use dioxus::prelude::*;
use uuid::Uuid;
use zwipe::{
    domain::{
        auth::models::session::Session,
        deck::models::deck::{copy_max::CopyMax, deck_profile::DeckProfile},
    },
    inbound::http::ApiError,
};

#[component]
pub fn GetDeck(deck_id: Uuid) -> Element {
    let swipe_state = use_signal(|| SwipeState::new());
    let swipe_config = SwipeConfig::blank();

    let navigator = use_navigator();

    let session: Signal<Option<Session>> = use_context();
    let auth_client: Signal<AuthClient> = use_context();

    let deck_profile_resource: Resource<Result<DeckProfile, ApiError>> =
        use_resource(move || async move {
            session.upkeep(auth_client);
            let Some(sesh) = session() else {
                return Err(ApiError::Unauthorized("session expired".to_string()));
            };

            auth_client().get_deck_profile(deck_id, &sesh).await
        });

    let commander_name_resource: Resource<Result<Option<String>, ApiError>> =
        use_resource(move || async move {
            let Some(Ok(profile)) = &*deck_profile_resource.read() else {
                return Ok(None);
            };

            let Some(commander_id) = profile.commander_id else {
                return Ok(None);
            };

            let Some(sesh) = session() else {
                return Err(ApiError::Unauthorized("session expired".to_string()));
            };

            auth_client()
                .get_card(&commander_id, &sesh)
                .await
                .map(|value| Some(value.scryfall_data.name))
        });

    rsx! {
        Bouncer {
            Swipeable { state: swipe_state, config: swipe_config,
                div { class : "form-container",

                match &*deck_profile_resource.read() {
                    Some(Ok(profile)) => rsx! {
                        h2 { "{profile.name}" }

                        form {
                            label { r#for : "commander", "" }
                            div { class: "commander-search",
                                input {
                                    id: "commander",
                                    r#type : "text",
                                    placeholder : "commander",
                                    value : {
                                        if let Some(Ok(Some(commander_name))) = &*commander_name_resource.read() {
                                            commander_name
                                        } else {
                                            "none"
                                        }
                                     },
                                    readonly : true,
                                }
                            }

                            label { r#for : "copy-max", "card copy rule" }
                            div {
                                class: "form-group-copy-max",
                                div {
                                    class: if profile.copy_max == Some(CopyMax::standard()) { "copy-max-box true" } else { "copy-max-box false" },
                                    "standard"
                                }
                                div {
                                    class: if profile.copy_max == Some(CopyMax::singleton()) { "copy-max-box true" } else { "copy-max-box false" },
                                    "singleton"
                                }
                                div {
                                    class: if profile.copy_max.is_none() { "copy-max-box true" } else { "copy-max-box false" },
                                    "none"
                                }
                            }

                            button {
                                onclick : move |_| {
                                    navigator.push(Router::UpdateDeck { deck_id: Uuid::new_v4() });
                                },
                                "update"
                            }

                            button {
                                onclick : move |_| {
                                    navigator.push(Router::DeckList {});
                                },
                                "back"
                            }
                        }
                        },
                        Some(Err(e)) => rsx! { div { class : "error", "{e}"} },
                        None => rsx! { div { class : "spinning-card" } }
                    }
                }
            }
        }
    }
}
