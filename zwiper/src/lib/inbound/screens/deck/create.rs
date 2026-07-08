//! Create new deck screen.

use super::components::{
    deck_fields::{DeckFields, DeckFieldsHint, autofill_named_partner},
    format_select::FormatSelect,
    swipe_select::{SwipeMode, SwipeSelect},
    tag_select::TagSelect,
};
use crate::{
    inbound::{
        components::{
            auth::{bouncer::Bouncer, ensure_session::EnsureFresh},
            hint_dialog::use_one_time_hint,
            screen_header::ScreenHeader,
        },
        router::Router,
    },
    outbound::client::{ZwipeClient, deck::create_deck::ClientCreateDeck},
};
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use std::time::Duration;
use zwipe_components::{ActionBar, Button, ButtonVariant};
use zwipe_core::{
    domain::{
        auth::models::session::Session,
        card::{Card, search_card::card_filter::price_currency::PriceCurrency},
        deck::{DeckName, DeckOtherTag, DeckTag, PowerLevel, format::Format},
        user::models::hints::HINT_CREATE_DECK,
    },
    http::contracts::deck::HttpCreateDeckProfile,
};

/// Screen for creating a new deck with name and settings.
#[component]
pub fn CreateDeck() -> Element {
    let navigator = use_navigator();

    let session: Signal<Option<Session>> = use_context();
    let auth_client: Signal<ZwipeClient> = use_context();

    // form
    let deck_name = use_signal(String::new);
    let mut deck_name_error: Signal<Option<String>> = use_signal(|| None);
    let mut selected_format: Signal<Option<Format>> = use_signal(|| None);
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
    let mut show_tags_select = use_signal(|| false);
    let mut show_format_select = use_signal(|| false);
    let land_target = use_signal(|| None::<i32>);
    let price_target = use_signal(String::new);
    let price_target_currency = use_signal(|| PriceCurrency::Usd);
    let power_level: Signal<Option<PowerLevel>> = use_signal(|| None);
    let other_tags: Signal<Vec<DeckOtherTag>> = use_signal(Vec::new);
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
        if let Err(e) = DeckName::new(deck_name()) {
            deck_name_error.set(Some(e.to_string()));
            return;
        }
        is_saving.set(true);

        spawn(async move {
            let session = match session.ensure_fresh(auth_client).await {
                Ok(session) => session,
                Err(e) => {
                    toast.error(
                        e.to_user_message(),
                        ToastOptions::default().duration(Duration::from_millis(3000)),
                    );
                    is_saving.set(false);
                    return;
                }
            };

            let commander_id = commander().map(|c| c.scryfall_data.id);
            let format_str = selected_format().map(|f| f.to_legality_key().to_string());
            let tags: Vec<String> = selected_tags().iter().map(|t| t.to_string()).collect();
            let other: Vec<String> = other_tags().iter().map(|t| t.to_string()).collect();
            let price_target_val: Option<f64> =
                price_target().parse().ok().filter(|v: &f64| *v > 0.0);
            let request = HttpCreateDeckProfile::builder(&deck_name())
                .commander_id(commander_id)
                .partner_commander_id(partner_commander().map(|c| c.scryfall_data.id))
                .background_id(background().map(|c| c.scryfall_data.id))
                .signature_spell_id(signature_spell().map(|c| c.scryfall_data.id))
                .format(format_str)
                .tags(if tags.is_empty() { None } else { Some(tags) })
                .power_level(power_level().map(|p| p.to_string()))
                .other_tags(if other.is_empty() { None } else { Some(other) })
                .land_target(land_target())
                .price_target(price_target_val)
                .price_target_currency(price_target_val.map(|_| price_target_currency()))
                .build();

            match auth_client().create_deck_profile(&request, &session).await {
                Ok(created) => {
                    navigator.push(Router::ViewDeck {
                        deck_id: created.id,
                    });
                }
                Err(e) => {
                    toast.error(
                        e.to_user_message(),
                        ToastOptions::default().duration(Duration::from_millis(3000)),
                    );
                    is_saving.set(false);
                }
            }
        });
    };

    rsx! {
        Bouncer {
            div { class: "screen",
                ScreenHeader { title: "Create Deck", hint: create_hint }

                div { class: "screen-content content-enter",
                div { class : "container-sm",

                    form { class : "flex-col text-center",
                        DeckFields {
                            deck_name,
                            deck_name_error,
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
                            show_tags_select,
                            show_format_select,
                            land_target,
                            price_target,
                            price_target_currency,
                            power_level,
                            other_tags,
                        }
                    }
                }
            }

            ActionBar {
                Button {
                    variant: ButtonVariant::Util,
                    disabled: is_saving(),
                    onclick: move |_| navigator.go_back(),
                    "Back"
                }
                Button {
                    variant: ButtonVariant::Util,
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
                    autofill_named_partner(
                        &card,
                        auth_client,
                        session,
                        commander,
                        partner_commander,
                        partner_commander_display,
                        toast,
                    );
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

            TagSelect {
                open: show_tags_select,
                selected_tags,
                on_close: move |_| show_tags_select.set(false),
            }

            FormatSelect {
                open: show_format_select,
                selected_format,
                on_select: move |fmt: Format| {
                    selected_format.set(Some(fmt));
                    commander.set(None);
                    commander_display.set(String::new());
                    if !fmt.has_signature_spell() {
                        signature_spell.set(None);
                        signature_spell_display.set(String::new());
                    }
                },
                on_clear: move |_| {
                    selected_format.set(None);
                    commander.set(None);
                    commander_display.set(String::new());
                    signature_spell.set(None);
                    signature_spell_display.set(String::new());
                },
                on_close: move |_| show_format_select.set(false),
            }

            DeckFieldsHint { open: create_hint }
        }
    }
}
