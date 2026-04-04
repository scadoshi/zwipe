//! Create new deck screen.

use super::components::deck_fields::DeckFields;
use crate::{
    inbound::{
        components::auth::{bouncer::Bouncer, session_upkeep::Upkeep},
        router::Router,
    },
    outbound::client::{deck::create_deck::ClientCreateDeck, ZwipeClient},
};
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use std::time::Duration;
use zwipe_core::domain::deck::format::Format;
use zwipe_core::http::contracts::deck::HttpCreateDeckProfile;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::card::Card;

/// Screen for creating a new deck with name and settings.
#[component]
pub fn CreateDeck() -> Element {
    let navigator = use_navigator();

    let session: Signal<Option<Session>> = use_context();
    let auth_client: Signal<ZwipeClient> = use_context();

    // form
    let deck_name = use_signal(String::new);
    let selected_format: Signal<Option<Format>> = use_signal(|| None);
    let commander: Signal<Option<Card>> = use_signal(|| None);
    let commander_display = use_signal(String::new);

    // save state
    let toast = use_toast();
    let mut is_saving = use_signal(|| false);

    let mut attempt_submit = move || {
        is_saving.set(true);

        spawn(async move {
            session.upkeep(auth_client);
            let Some(session) = session() else {
                toast.error("session expired".to_string(), ToastOptions::default().duration(Duration::from_millis(3000)));
                is_saving.set(false);
                return;
            };

            let commander_id = commander().map(|c| c.scryfall_data.id);
            let format_str = selected_format().map(|f| f.to_legality_key().to_string());
            let request = HttpCreateDeckProfile::builder(&deck_name())
                .commander_id(commander_id)
                .format(format_str)
                .build();

            match auth_client().create_deck_profile(&request, &session).await {
                Ok(created) => {
                    navigator.push(Router::ViewDeck {
                        deck_id: created.id,
                    });
                }
                Err(e) => {
                    toast.error(e.to_string().to_lowercase(), ToastOptions::default().duration(Duration::from_millis(3000)));
                    is_saving.set(false);
                }
            }
        });
    };

    rsx! {
        Bouncer {
            div { class: "screen",
                div { class: "page-header",
                    h2 { "create deck" }
                }

                div { class: "screen-content centered content-enter",
                div { class : "container-sm",

                    form { class : "flex-col text-center",
                        DeckFields {
                            deck_name,
                            selected_format,
                            commander,
                            commander_display,
                        }
                    }
                }
            }

            div { class: "util-bar",
                button {
                    class: "util-btn",
                    onclick: move |_| navigator.go_back(),
                    "back"
                }
                button { class : "util-btn",
                    disabled: is_saving(),
                    onclick : move |_| attempt_submit(),
                    if is_saving() { "saving..." } else { "save" }
                }
            }
            }
        }
    }
}
