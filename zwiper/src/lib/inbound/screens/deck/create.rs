//! Create new deck screen.

use super::components::deck_fields::{DeckFields, DeckFieldsHint};
use super::components::swipe_select::{SwipeMode, SwipeSelect};
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
use crate::inbound::components::screen_header::ScreenHeader;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use std::time::Duration;
use zwipe_core::domain::deck::{DeckTag, format::Format};
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
    let selected_tags: Signal<Vec<DeckTag>> = use_signal(Vec::new);
    let mut commander: Signal<Option<Card>> = use_signal(|| None);
    let mut commander_display = use_signal(String::new);
    let mut partner_commander: Signal<Option<Card>> = use_signal(|| None);
    let mut partner_commander_display = use_signal(String::new);
    let mut background: Signal<Option<Card>> = use_signal(|| None);
    let mut background_display = use_signal(String::new);
    let mut signature_spell: Signal<Option<Card>> = use_signal(|| None);
    let mut signature_spell_display = use_signal(String::new);
    let mut show_commander_swipe = use_signal(|| false);
    let mut show_partner_swipe = use_signal(|| false);
    let mut show_background_swipe = use_signal(|| false);
    let mut show_signature_spell_swipe = use_signal(|| false);
    let create_hint = use_one_time_hint(HINT_CREATE_DECK);

    // Reactive Zwipe-select modes — derived from the current format / commander.
    let commander_mode = use_memo(move || selected_format().map(SwipeMode::Commander));
    let partner_mode = use_memo(|| Some(SwipeMode::Partner));
    let background_mode = use_memo(|| Some(SwipeMode::Background));
    let spell_mode = use_memo(move || {
        commander().map(|c| SwipeMode::SignatureSpell(c.scryfall_data.color_identity))
    });

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
            let tags: Vec<String> = selected_tags().iter().map(|t| t.to_string()).collect();
            let request = HttpCreateDeckProfile::builder(&deck_name())
                .commander_id(commander_id)
                .partner_commander_id(partner_commander().map(|c| c.scryfall_data.id))
                .background_id(background().map(|c| c.scryfall_data.id))
                .signature_spell_id(signature_spell().map(|c| c.scryfall_data.id))
                .format(format_str)
                .tags(if tags.is_empty() { None } else { Some(tags) })
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
                ScreenHeader { title: "Create Deck", hint: create_hint }

                div { class: "screen-content centered content-enter",
                div { class : "container-sm",

                    form { class : "flex-col text-center",
                        DeckFields {
                            deck_name,
                            selected_format,
                            selected_tags,
                            commander,
                            commander_display,
                            partner_commander,
                            partner_commander_display,
                            background,
                            background_display,
                            signature_spell,
                            signature_spell_display,
                            show_commander_swipe,
                            show_partner_swipe,
                            show_background_swipe,
                            show_signature_spell_swipe,
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
            SwipeSelect {
                open: show_commander_swipe,
                mode: commander_mode,
                on_select: move |card: Card| {
                    commander_display.set(card.scryfall_data.name.clone());
                    commander.set(Some(card));
                    show_commander_swipe.set(false);
                },
                on_close: move |_| show_commander_swipe.set(false),
            }
            SwipeSelect {
                open: show_partner_swipe,
                mode: partner_mode,
                on_select: move |card: Card| {
                    partner_commander_display.set(card.scryfall_data.name.clone());
                    partner_commander.set(Some(card));
                    show_partner_swipe.set(false);
                },
                on_close: move |_| show_partner_swipe.set(false),
            }
            SwipeSelect {
                open: show_background_swipe,
                mode: background_mode,
                on_select: move |card: Card| {
                    background_display.set(card.scryfall_data.name.clone());
                    background.set(Some(card));
                    show_background_swipe.set(false);
                },
                on_close: move |_| show_background_swipe.set(false),
            }
            SwipeSelect {
                open: show_signature_spell_swipe,
                mode: spell_mode,
                on_select: move |card: Card| {
                    signature_spell_display.set(card.scryfall_data.name.clone());
                    signature_spell.set(Some(card));
                    show_signature_spell_swipe.set(false);
                },
                on_close: move |_| show_signature_spell_swipe.set(false),
            }

            DeckFieldsHint { open: create_hint }
        }
    }
}
