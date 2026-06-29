//! Edit deck screen.

use super::components::deck_fields::{DeckFields, DeckFieldsHint};
use super::components::format_select::FormatSelect;
use super::components::tag_select::TagSelect;
use super::components::skeletons::EditDeckSkeleton;
use super::components::swipe_select::{SwipeMode, SwipeSelect};
use crate::inbound::components::screen_header::ScreenHeader;
use crate::{
    inbound::{
        components::auth::{bouncer::Bouncer, ensure_session::EnsureFresh},
        components::hint_dialog::use_one_time_hint,
        router::Router,
    },
    outbound::client::{
        ZwipeClient,
        card::get_card::ClientGetCard,
        deck::{get_deck::ClientGetDeck, update_deck_profile::ClientUpdateDeckProfile},
    },
};
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use std::time::Duration;
use uuid::Uuid;
use zwipe::inbound::http::ApiError;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::deck::{
    Deck, DeckTag, deck_profile::DeckProfile, format::Format,
    requests::update_deck_profile::InvalidUpdateDeckProfile,
};
use zwipe_core::domain::user::models::hints::HINT_EDIT_DECK;
use zwipe_core::http::contracts::deck::HttpUpdateDeckProfile;
use zwipe_core::http::helpers::Opdate;

/// Screen for editing a deck with name and settings.
#[component]
pub fn EditDeck(deck_id: Uuid) -> Element {
    // config
    let navigator = use_navigator();
    let session: Signal<Option<Session>> = use_context();
    let client: Signal<ZwipeClient> = use_context();

    // current values
    let mut deck_name: Signal<String> = use_signal(String::new);
    let mut commander: Signal<Option<Card>> = use_signal(|| None);
    let mut commander_display = use_signal(String::new);
    let mut show_commander_swipe = use_signal(|| false);
    let edit_hint = use_one_time_hint(HINT_EDIT_DECK);
    let mut selected_format: Signal<Option<Format>> = use_signal(|| None);
    let mut selected_tags: Signal<Vec<DeckTag>> = use_signal(Vec::new);
    let mut partner_commander: Signal<Option<Card>> = use_signal(|| None);
    let mut partner_commander_display = use_signal(String::new);
    let mut background: Signal<Option<Card>> = use_signal(|| None);
    let mut background_display = use_signal(String::new);
    let mut signature_spell: Signal<Option<Card>> = use_signal(|| None);
    let mut signature_spell_display = use_signal(String::new);
    let mut show_partner_swipe = use_signal(|| false);
    let mut show_background_swipe = use_signal(|| false);
    let mut show_signature_spell_swipe = use_signal(|| false);
    let mut show_tags_select = use_signal(|| false);
    let mut show_format_select = use_signal(|| false);

    // Reactive Zwipe-select modes — derived from the current format / commander.
    let commander_mode = use_memo(move || selected_format().map(SwipeMode::Commander));
    let partner_mode = use_memo(|| Some(SwipeMode::Partner));
    let background_mode = use_memo(|| Some(SwipeMode::Background));
    let spell_mode = use_memo(move || {
        commander().map(|c| SwipeMode::SignatureSpell(c.scryfall_data.color_identity))
    });

    // original values (for change detection)
    let mut original_deck_name: Signal<String> = use_signal(String::new);
    let mut original_commander: Signal<Option<Card>> = use_signal(|| None);
    let mut original_format: Signal<Option<Format>> = use_signal(|| None);
    let mut original_tags: Signal<Vec<DeckTag>> = use_signal(Vec::new);
    let mut original_partner: Signal<Option<Card>> = use_signal(|| None);
    let mut original_background: Signal<Option<Card>> = use_signal(|| None);
    let mut original_signature_spell: Signal<Option<Card>> = use_signal(|| None);

    let toast = use_toast();

    // ========================================
    // Fetch deck profile
    // ========================================
    let original_deck_resource: Resource<Result<Deck, ApiError>> =
        use_resource(move || async move {
            let session = session.ensure_fresh(client).await?;
            client().get_deck(deck_id, &session).await
        });
    use_effect(move || match original_deck_resource() {
        Some(Ok(deck)) => {
            original_deck_name.set(deck.deck_profile.name.to_string());
            deck_name.set(deck.deck_profile.name.to_string());
            original_format.set(deck.deck_profile.format);
            selected_format.set(deck.deck_profile.format);
            original_tags.set(deck.deck_profile.tags.clone());
            selected_tags.set(deck.deck_profile.tags);
        }
        Some(Err(e)) => {
            toast.error(
                e.to_user_message(),
                ToastOptions::default().duration(Duration::from_millis(3000)),
            );
        }
        None => (),
    });

    // ========================================
    // Fetch commander card
    // ========================================
    let original_commander_resource: Resource<Result<Option<Card>, ApiError>> =
        use_resource(move || async move {
            let Some(Ok(Deck {
                deck_profile:
                    DeckProfile {
                        commander_id: Some(original_commander_id),
                        ..
                    },
                ..
            })) = original_deck_resource()
            else {
                return Ok(None);
            };
            client().get_card(original_commander_id).await.map(Some)
        });
    use_effect(move || match original_commander_resource() {
        Some(Ok(Some(original))) => {
            original_commander.set(Some(original.clone()));
            commander.set(Some(original.clone()));
            commander_display.set(original.scryfall_data.name);
        }
        Some(Ok(None)) | None => (),
        Some(Err(e)) => {
            toast.error(
                e.to_user_message(),
                ToastOptions::default().duration(Duration::from_millis(3000)),
            );
        }
    });

    // ========================================
    // Fetch partner commander card
    // ========================================
    let original_partner_resource: Resource<Result<Option<Card>, ApiError>> =
        use_resource(move || async move {
            let Some(Ok(Deck {
                deck_profile:
                    DeckProfile {
                        partner_commander_id: Some(id),
                        ..
                    },
                ..
            })) = original_deck_resource()
            else {
                return Ok(None);
            };
            client().get_card(id).await.map(Some)
        });
    use_effect(move || match original_partner_resource() {
        Some(Ok(Some(original))) => {
            original_partner.set(Some(original.clone()));
            partner_commander.set(Some(original.clone()));
            partner_commander_display.set(original.scryfall_data.name);
        }
        Some(Ok(None)) | None => (),
        Some(Err(e)) => {
            toast.error(
                e.to_user_message(),
                ToastOptions::default().duration(Duration::from_millis(3000)),
            );
        }
    });

    // ========================================
    // Fetch background card
    // ========================================
    let original_background_resource: Resource<Result<Option<Card>, ApiError>> =
        use_resource(move || async move {
            let Some(Ok(Deck {
                deck_profile:
                    DeckProfile {
                        background_id: Some(id),
                        ..
                    },
                ..
            })) = original_deck_resource()
            else {
                return Ok(None);
            };
            client().get_card(id).await.map(Some)
        });
    use_effect(move || match original_background_resource() {
        Some(Ok(Some(original))) => {
            original_background.set(Some(original.clone()));
            background.set(Some(original.clone()));
            background_display.set(original.scryfall_data.name);
        }
        Some(Ok(None)) | None => (),
        Some(Err(e)) => {
            toast.error(
                e.to_user_message(),
                ToastOptions::default().duration(Duration::from_millis(3000)),
            );
        }
    });

    // ========================================
    // Fetch signature spell card
    // ========================================
    let original_spell_resource: Resource<Result<Option<Card>, ApiError>> =
        use_resource(move || async move {
            let Some(Ok(Deck {
                deck_profile:
                    DeckProfile {
                        signature_spell_id: Some(id),
                        ..
                    },
                ..
            })) = original_deck_resource()
            else {
                return Ok(None);
            };
            client().get_card(id).await.map(Some)
        });
    use_effect(move || match original_spell_resource() {
        Some(Ok(Some(original))) => {
            original_signature_spell.set(Some(original.clone()));
            signature_spell.set(Some(original.clone()));
            signature_spell_display.set(original.scryfall_data.name);
        }
        Some(Ok(None)) | None => (),
        Some(Err(e)) => {
            toast.error(
                e.to_user_message(),
                ToastOptions::default().duration(Duration::from_millis(3000)),
            );
        }
    });

    // ========================================
    // Change tracking
    // ========================================
    let deck_name_update = use_memo(move || {
        if deck_name() != original_deck_name() {
            Some(deck_name())
        } else {
            None
        }
    });
    let commander_id_update = use_memo(move || {
        if commander() != original_commander() {
            Opdate::Set(commander().map(|c| c.scryfall_data.id))
        } else {
            Opdate::Unchanged
        }
    });
    let partner_id_update = use_memo(move || {
        if partner_commander() != original_partner() {
            Opdate::Set(partner_commander().map(|c| c.scryfall_data.id))
        } else {
            Opdate::Unchanged
        }
    });
    let background_id_update = use_memo(move || {
        if background() != original_background() {
            Opdate::Set(background().map(|c| c.scryfall_data.id))
        } else {
            Opdate::Unchanged
        }
    });
    let signature_spell_id_update = use_memo(move || {
        if signature_spell() != original_signature_spell() {
            Opdate::Set(signature_spell().map(|c| c.scryfall_data.id))
        } else {
            Opdate::Unchanged
        }
    });
    let format_update = use_memo(move || {
        if selected_format() != original_format() {
            Opdate::Set(selected_format().map(|f| f.to_legality_key().to_string()))
        } else {
            Opdate::Unchanged
        }
    });
    let tags_update = use_memo(move || {
        if selected_tags() != original_tags() {
            Opdate::Set(Some(
                selected_tags()
                    .iter()
                    .map(|t| t.to_string())
                    .collect::<Vec<_>>(),
            ))
        } else {
            Opdate::Unchanged
        }
    });
    let has_made_changes = use_memo(move || {
        deck_name_update().is_some()
            || commander_id_update().is_changed()
            || partner_id_update().is_changed()
            || background_id_update().is_changed()
            || signature_spell_id_update().is_changed()
            || format_update().is_changed()
            || tags_update().is_changed()
    });

    // save state
    let mut is_saving = use_signal(|| false);

    let mut do_submit = move || {
        is_saving.set(true);

        spawn(async move {
            let session = match session.ensure_fresh(client).await {
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

            if !has_made_changes() {
                toast.error(
                    InvalidUpdateDeckProfile::NoUpdates.to_string(),
                    ToastOptions::default().duration(Duration::from_millis(3000)),
                );
                is_saving.set(false);
                return;
            }

            let request = HttpUpdateDeckProfile::builder()
                .name(deck_name_update().as_deref())
                .commander_id(commander_id_update())
                .partner_commander_id(partner_id_update())
                .background_id(background_id_update())
                .signature_spell_id(signature_spell_id_update())
                .format(format_update())
                .tags(tags_update())
                .build();

            match client()
                .update_deck_profile(deck_id, &request, &session)
                .await
            {
                Ok(_updated) => {
                    is_saving.set(false);
                    navigator.push(Router::ViewDeck { deck_id });
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

    let mut attempt_submit = move || {
        do_submit();
    };

    rsx! {
        Bouncer {
            div { class: "screen",
                ScreenHeader { title: "Edit Deck", hint: edit_hint }

                div { class: "screen-content centered content-enter",
                div { class : "container-sm",
                    match &*original_deck_resource.read() {
                        Some(Ok(_deck)) => rsx! {
                            form { class: "flex-col text-center",
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
                                    show_tags_select,
                                    show_format_select,
                                }

                            }

                        },
                        Some(Err(_)) => rsx! { p { class: "text-muted", "Could not load deck" } },
                        None => rsx! { EditDeckSkeleton {} }
                    }
                }
            }

            div { class: "util-bar",
                button {
                    class: "util-btn",
                    disabled: is_saving(),
                    onclick: move |_| {
                        navigator.push(Router::ViewDeck { deck_id });
                    },
                    "Back"
                }
                if has_made_changes() {
                    button {
                        class: "util-btn",
                        disabled: is_saving(),
                        onclick : move |_| attempt_submit(),
                            if is_saving() { "Saving..." } else { "Save changes" }
                    }
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

            DeckFieldsHint { open: edit_hint }
        }
    }
}
