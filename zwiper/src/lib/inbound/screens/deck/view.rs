use crate::inbound::components::alert_dialog::{
    AlertDialogAction, AlertDialogActions, AlertDialogCancel, AlertDialogContent,
    AlertDialogDescription, AlertDialogRoot, AlertDialogTitle,
};
use crate::{
    inbound::{
        components::auth::{bouncer::Bouncer, session_upkeep::Upkeep},
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
        card::models::Card,
        deck::models::deck::{copy_max::CopyMax, deck_profile::DeckProfile},
    },
    inbound::http::ApiError,
};

#[component]
pub fn ViewDeck(deck_id: Uuid) -> Element {
    // config
    let navigator = use_navigator();
    let session: Signal<Option<Session>> = use_context();
    let client: Signal<ZwipeClient> = use_context();

    // original deck information
    let mut commander: Signal<Option<Card>> = use_signal(|| None);
    let mut load_error = use_signal(|| None::<String>);

    let deck_profile_resource: Resource<Result<DeckProfile, ApiError>> =
        use_resource(move || async move {
            session.upkeep(client);
            let Some(session) = session() else {
                return Err(ApiError::Unauthorized("session expired".to_string()));
            };

            client().get_deck_profile(deck_id, &session).await
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
            let Some(session) = session() else {
                return Err(ApiError::Unauthorized("session expired".to_string()));
            };
            client()
                .get_card(original_commander_id, &session)
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

    let mut show_delete_dialog = use_signal(|| false);
    let mut delete_error = use_signal(|| None::<String>);
    let mut attempt_delete = move || {
        session.upkeep(client);
        let Some(session) = session() else {
            delete_error.set(Some("session expired".to_string()));
            return;
        };

        spawn(async move {
            match client().delete_deck(deck_id, &session).await {
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
            div { class: "screen",
                div {
                    class : "page-header",
                    h2 { "deck" }
                }

                div { class: "screen-content centered",
                    match deck_profile_resource() {
                        Some(Ok(deck_profile)) => rsx! {
                            div { class: "container-sm",
                                if let Some(error) = load_error() {
                                    div { class: "message-error", "{error}" }
                                }

                                div { class: "flex items-center flex-between mb-4 gap-2",
                                    div { class: "flex-1",
                                        label { class: "label", "deck name" }
                                        p { class: "text-base font-light mb-1", "{deck_profile.name}" }
                                    }
                                }

                                div { class: "flex items-center flex-between mb-4 gap-2",
                                    div { class: "flex-1",
                                        label { class: "label", "copy rule" }
                                        p { class: "text-base font-light mb-1",
                                            if deck_profile.copy_max == Some(CopyMax::standard()) {
                                                "standard"
                                            } else if deck_profile.copy_max == Some(CopyMax::singleton()) {
                                                "singleton"
                                            } else {
                                                "none"
                                            }
                                        }
                                    }
                                }

                                div { class: "flex items-center flex-between mb-4 gap-2",
                                    div { class: "flex-1",
                                        label { class: "label", "commander" }
                                        p { class: "text-base font-light mb-1",
                                            if let Some(cmd) = commander() {
                                                { cmd.scryfall_data.name.to_lowercase() }
                                            } else {
                                                "none"
                                            }
                                        }
                                    }
                                }

                                if let Some(error) = delete_error() {
                                    div { class: "message-error", "{error}" }
                                }
                            }
                        },
                        Some(Err(e)) => rsx! { div { class: "message-error", "{e}" } },
                        None => rsx! { div { class: "spinner" } }
                    }
                }

            div { class: "util-bar",
                button {
                    class: "util-btn",
                    onclick: move |_| {
                        navigator.push(Router::DeckList {});
                    },
                    "back"
                }
                button {
                    class : "util-btn",
                    onclick : move |_| {
                        navigator.push(Router::ViewDeckCard { deck_id });
                    },
                    "view"
                }
                button {
                    class : "util-btn",
                    onclick : move |_| {
                        navigator.push(Router::AddDeckCard { deck_id });
                    },
                    "add"
                }
                button {
                    class : "util-btn",
                    onclick : move |_| {
                        navigator.push(Router::RemoveDeckCard { deck_id });
                    },
                    "remove"
                }
                button {
                    class: "util-btn",
                    onclick: move |_| {
                        navigator.push(Router::EditDeck { deck_id });
                    },
                    "edit"
                }
                button {
                    class: "util-btn",
                    onclick: move |_| show_delete_dialog.set(true),
                    "delete"
                }
            }

            AlertDialogRoot {
                open: show_delete_dialog(),
                on_open_change: move |open| show_delete_dialog.set(open),
                AlertDialogContent {
                    AlertDialogTitle { "delete deck" }
                    AlertDialogDescription { "are you sure you want to delete this deck?" }
                    AlertDialogActions {
                        AlertDialogCancel {
                            on_click: move |_| show_delete_dialog.set(false),
                            "cancel"
                        }
                        AlertDialogAction {
                            on_click: move |_| attempt_delete(),
                            "delete"
                        }
                    }
                }
            }
            }
        }
    }
}
