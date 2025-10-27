use crate::{
    inbound::ui::{
        components::{
            auth::{bouncer::Bouncer, session_upkeep::Upkeep},
            interactions::swipe::{config::SwipeConfig, state::SwipeState, Swipeable},
        },
        router::Router,
    },
    outbound::client::{
        card::get_card::ClientGetCard,
        deck::{delete_deck::ClientDeleteDeck, get_deck_profile::ClientGetDeckProfile},
        ZwipeClient,
    },
};
use dioxus::prelude::*;
use uuid::Uuid;
use zwipe::{
    domain::{
        auth::models::session::Session,
        card::models::{scryfall_data::image_uris::ImageUris, Card},
        deck::models::deck::{copy_max::CopyMax, deck_profile::DeckProfile},
    },
    inbound::http::ApiError,
};

#[component]
pub fn ViewDeckProfile(deck_id: Uuid) -> Element {
    let swipe_state = use_signal(|| SwipeState::new());
    let swipe_config = SwipeConfig::blank();

    let navigator = use_navigator();

    let session: Signal<Option<Session>> = use_context();
    let client: Signal<ZwipeClient> = use_context();

    let deck_profile_resource: Resource<Result<DeckProfile, ApiError>> =
        use_resource(move || async move {
            session.upkeep(client);
            let Some(sesh) = session() else {
                return Err(ApiError::Unauthorized("session expired".to_string()));
            };

            client().get_deck_profile(deck_id, &sesh).await
        });

    let commander_resource: Resource<Result<Option<Card>, ApiError>> =
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

            client()
                .get_card(&commander_id, &sesh)
                .await
                .map(|value| Some(value))
        });

    let mut show_delete_confirmation = use_signal(|| false);
    let mut delete_error = use_signal(|| None::<String>);
    let mut attempt_delete = move || {
        session.upkeep(client);
        let Some(sesh) = session() else {
            delete_error.set(Some("session expired".to_string()));
            return;
        };

        spawn(async move {
            match client().delete_deck(&deck_id, &sesh).await {
                Ok(_) => {
                    navigator.push(Router::DeckList {});
                }
                Err(e) => {
                    delete_error.set(Some(e.to_string()));
                }
            }
        });
    };

    rsx! {
    Bouncer {
        Swipeable { state: swipe_state, config: swipe_config,
            div { class : "form-container",
                match &*deck_profile_resource.read() {
                    Some(Ok(profile)) => rsx! {

                        h2 { "{profile.name}" }
                        div { class : "form-group",

                            if let Some(Ok(Some(commander))) = &*commander_resource.read() {
                                if let Some(ImageUris { normal: Some(image_url), .. }) = &commander.scryfall_data.image_uris {
                                    img {
                                        src: "{image_url}",
                                        alt: "{commander.scryfall_data.name}",
                                        class: "commander-image"
                                    }
                                } else {
                                    label { r#for : "commander-info", "commander" }
                                    p { class: "commander-name-only", { commander.scryfall_data.name.to_lowercase() } }
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

                            if !show_delete_confirmation() {
                                button {
                                    onclick : move |_| {
                                        navigator.push(Router::EditDeckProfile { deck_id });
                                    },
                                    "edit"
                                }

                                button { class : "delete-button",
                                    onclick : move |_| show_delete_confirmation.set(true),
                                    "delete"
                                }
                            }

                            if show_delete_confirmation() {
                                label { r#for : "confirmation-prompt", "are you sure?" }
                                div { class : "confirmation-prompt",
                                    id : "confirmation-prompt",
                                    button { class : "yes-button",
                                        onclick : move |_| attempt_delete(),
                                        "yes"
                                    }
                                    button { class : "no-button",
                                        onclick : move |_| show_delete_confirmation.set(false),
                                        "no"
                                    }
                                }
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
