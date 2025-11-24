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
pub fn ViewDeck(deck_id: Uuid) -> Element {
    // config
    let swipe_state = use_signal(SwipeState::new);
    let swipe_config = SwipeConfig::blank();
    let navigator = use_navigator();
    let session: Signal<Option<Session>> = use_context();
    let client: Signal<ZwipeClient> = use_context();

    // original deck information
    let mut commander: Signal<Option<Card>> = use_signal(|| None);
    let mut load_error = use_signal(|| None::<String>);

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
            let Some(Ok(DeckProfile {
                commander_id: Some(original_commander_id),
                ..
            })) = deck_profile_resource()
            else {
                return Ok(None);
            };
            session.upkeep(client);
            let Some(sesh) = session() else {
                return Err(ApiError::Unauthorized("session expired".to_string()));
            };
            client()
                .get_card(original_commander_id, &sesh)
                .await
                .map(Some)
        });
    use_effect(move || match commander_resource() {
        Some(Ok(Some(original_commander))) => {
            commander.set(Some(original_commander));
        }
        Some(Err(e)) => {
            load_error.set(Some(e.to_string()));
        }
        Some(Ok(None)) | None => (),
    });

    let mut confirm_deletion = use_signal(|| false);
    let mut delete_error = use_signal(|| None::<String>);
    let mut attempt_delete = move || {
        session.upkeep(client);
        let Some(sesh) = session() else {
            delete_error.set(Some("session expired".to_string()));
            return;
        };

        spawn(async move {
            match client().delete_deck(deck_id, &sesh).await {
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
                div { class : "container-sm text-center",
                    match deck_profile_resource() {
                        Some(Ok(deck_profile)) => rsx! {
                            h2 { class: "text-center mb-2 font-light tracking-wider", "{deck_profile.name}" }

                            if let Some(error) = load_error() {
                                div { class: "message-error", "{error}" }
                            }

                            div { class : "flex-col",

                                if let Some(cmd) = commander() {
                                    if let Some(ImageUris { normal: Some(image_url), .. }) = &cmd.scryfall_data.image_uris {
                                        img { class: "card-image",
                                            src: "{image_url}",
                                            alt: "{cmd.scryfall_data.name}",
                                        }
                                    } else {
                                        label { class: "label", r#for : "commander-info", "commander" }
                                        p { class: "text-center text-base font-light mb-4 tracking-wide",
                                            { cmd.scryfall_data.name.to_lowercase() }
                                        }
                                    }
                                }

                                label { class: "label", r#for : "copy-max", "card copy rule" }
                                p { class: "text-base font-light mb-4",
                                    if deck_profile.copy_max == Some(CopyMax::standard()) {
                                        "standard"
                                    } else if deck_profile.copy_max == Some(CopyMax::singleton()) {
                                        "singleton"
                                    } else {
                                        "none"
                                    }
                                }

                                if !confirm_deletion() {
                                    div { class : "flex flex-between gap-2",
                                        id : "confirmation-prompt",
                                        button { class: "btn btn-half",
                                            onclick : move |_| {
                                                navigator.push(Router::EditDeck { deck_id });
                                            },
                                            "edit"
                                        }
                                        button { class : "btn btn-half",
                                            onclick : move |_| confirm_deletion.set(true),
                                            "delete"
                                        }
                                    }
                                }

                                if confirm_deletion() {
                                    label { class: "label", r#for : "confirmation-prompt", "are you sure?" }
                                    div { class : "flex flex-between gap-2",
                                        id : "confirmation-prompt",
                                        button { class : "btn btn-half",
                                            onclick : move |_| attempt_delete(),
                                            "yes"
                                        }
                                        button { class : "btn btn-half",
                                            onclick : move |_| confirm_deletion.set(false),
                                            "no"
                                        }
                                    }
                                }

                                if let Some(error) = delete_error() {
                                    div { class: "message-error", "{error}" }
                                }

                                button { class: "btn",
                                    onclick : move |_| {
                                        navigator.push(Router::DeckList {} );
                                    },
                                    "back"
                                }
                            }
                        },
                    Some(Err(e)) => rsx! { div { class : "message-error", "{e}"} },
                    None => rsx! { div { class : "spinner" } }
                    }
                }
            }
        }
    }
}
