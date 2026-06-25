//! Create new deck screen.

use super::components::commander_swipe::CommanderSwipe;
use super::components::deck_fields::{DeckFields, DeckFieldsHint};
use crate::{
    inbound::{
        components::auth::{bouncer::Bouncer, ensure_session::EnsureFresh},
        components::hint_dialog::use_one_time_hint,
        router::Router,
    },
    outbound::client::{deck::create_deck::ClientCreateDeck, ZwipeClient},
};
use zwipe_core::domain::user::models::hints::HINT_CREATE_DECK;
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
    let mut commander: Signal<Option<Card>> = use_signal(|| None);
    let mut commander_display = use_signal(String::new);
    let partner_commander: Signal<Option<Card>> = use_signal(|| None);
    let partner_commander_display = use_signal(String::new);
    let background: Signal<Option<Card>> = use_signal(|| None);
    let background_display = use_signal(String::new);
    let signature_spell: Signal<Option<Card>> = use_signal(|| None);
    let signature_spell_display = use_signal(String::new);
    let mut show_commander_swipe = use_signal(|| false);
    let mut create_hint = use_one_time_hint(HINT_CREATE_DECK);

    // save state
    let toast = use_toast();
    let mut is_saving = use_signal(|| false);

    let mut attempt_submit = move || {
        is_saving.set(true);

        spawn(async move {
            let session = match session.ensure_fresh(auth_client).await {
                Ok(session) => session,
                Err(e) => {
                    toast.error(e.to_user_message(), ToastOptions::default().duration(Duration::from_millis(3000)));
                    is_saving.set(false);
                    return;
                }
            };

            let commander_id = commander().map(|c| c.scryfall_data.id);
            let format_str = selected_format().map(|f| f.to_legality_key().to_string());
            let request = HttpCreateDeckProfile::builder(&deck_name())
                .commander_id(commander_id)
                .partner_commander_id(partner_commander().map(|c| c.scryfall_data.id))
                .background_id(background().map(|c| c.scryfall_data.id))
                .signature_spell_id(signature_spell().map(|c| c.scryfall_data.id))
                .format(format_str)
                .build();

            match auth_client().create_deck_profile(&request, &session).await {
                Ok(created) => {
                    navigator.push(Router::ViewDeck {
                        deck_id: created.id,
                    });
                }
                Err(e) => {
                    toast.error(e.to_user_message(), ToastOptions::default().duration(Duration::from_millis(3000)));
                    is_saving.set(false);
                }
            }
        });
    };

    rsx! {
        Bouncer {
            div { class: "screen",
                div { class: "page-header", style: "position: relative;",
                    h2 { "Create Deck" }
                    button {
                        class: "util-btn",
                        style: "position: absolute; right: 1rem; top: 50%; transform: translateY(-50%); opacity: 0.55; padding: 0.2rem 0.6rem;",
                        onclick: move |_| create_hint.set(true),
                        "?"
                    }
                }

                div { class: "screen-content centered content-enter",
                div { class : "container-sm",

                    form { class : "flex-col text-center",
                        DeckFields {
                            deck_name,
                            selected_format,
                            commander,
                            commander_display,
                            partner_commander,
                            partner_commander_display,
                            background,
                            background_display,
                            signature_spell,
                            signature_spell_display,
                            show_commander_swipe,
                        }
                    }
                }
            }

            div { class: "util-bar",
                button {
                    class: "util-btn",
                    disabled: is_saving(),
                    onclick: move |_| navigator.go_back(),
                    "Back"
                }
                button { class : "util-btn",
                    disabled: is_saving(),
                    onclick : move |_| attempt_submit(),
                    if is_saving() { "Saving..." } else { "Save" }
                }
            }
            }
            CommanderSwipe {
                open: show_commander_swipe,
                format: selected_format,
                on_select: move |card: Card| {
                    commander_display.set(card.scryfall_data.name.clone());
                    commander.set(Some(card));
                    show_commander_swipe.set(false);
                },
                on_close: move |_| show_commander_swipe.set(false),
            }

            DeckFieldsHint { open: create_hint }
        }
    }
}
