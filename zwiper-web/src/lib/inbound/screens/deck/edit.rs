//! Edit deck: the deck form filled from the saved deck, with `Save changes`
//! only while something differs from it. Only the changed fields are sent.

use components::{ActionBar, Button, ButtonVariant, TOAST_NORMAL};
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use uuid::Uuid;
use zwipe_client::{ClientError, ZwipeClient};
use zwipe_core::domain::deck::requests::update_deck_profile::InvalidUpdateDeckProfile;
use zwipe_core::domain::deck::{Deck, DeckName};
use zwipe_ui::deck::form::Slot;

use super::form::fields::DeckFields;
use super::form::overlays::DeckFormOverlays;
use super::form::use_deck_form;
use super::skeletons::EditDeckSkeleton;
use crate::domain::deck::{FormValues, update_request};
use crate::inbound::chrome::authed::use_authed;
use crate::inbound::chrome::one_time_hint::{OneTimeHint, OneTimeHintDialog, use_one_time_hint};
use crate::inbound::router::Route;
use crate::inbound::shell::ScreenFrame;

/// The Edit deck screen for `deck_id`.
#[component]
pub fn EditDeck(deck_id: Uuid) -> Element {
    let navigator = use_navigator();
    let client: Signal<ZwipeClient> = use_context();
    let authed = use_authed();
    let toast = use_toast();
    let hint_open = use_one_time_hint(OneTimeHint::EditDeck);
    let form = use_deck_form();
    let mut original: Signal<FormValues> = use_signal(FormValues::default);
    let mut saving = use_signal(|| false);

    let deck: Resource<Result<Deck, ClientError>> = use_resource(move || async move {
        authed
            .try_run("load_deck", |c, s| async move {
                c.fetch_deck(deck_id, &s).await
            })
            .await
    });

    // The profile fills the form at once; each command-zone card fills its
    // slot, and its original, as it arrives, in any order.
    use_effect(move || {
        let Some(Ok(loaded)) = &*deck.read() else {
            return;
        };
        let profile = loaded.deck_profile.clone();
        form.fill(&profile);
        original.set(FormValues::saved(&profile));
        let slots = [
            (Slot::Commander, profile.commander_id),
            (Slot::Partner, profile.partner_commander_id),
            (Slot::Background, profile.background_id),
            (Slot::SignatureSpell, profile.signature_spell_id),
        ];
        for (slot, id) in slots {
            let Some(id) = id else {
                continue;
            };
            spawn(async move {
                let http = client.peek().clone();
                match http.fetch_card(id).await {
                    Ok(card) => {
                        let mut held = original.write();
                        match slot {
                            Slot::Commander => held.commander = Some(id),
                            Slot::Partner => held.partner = Some(id),
                            Slot::Background => held.background = Some(id),
                            Slot::SignatureSpell => held.signature_spell = Some(id),
                        }
                        drop(held);
                        form.slot(slot).pick(card);
                    }
                    Err(e) => {
                        tracing::warn!("load {slot:?} failed: {e}");
                        toast.error(
                            e.to_user_message(),
                            ToastOptions::default().duration(TOAST_NORMAL),
                        );
                    }
                }
            });
        }
    });

    let has_changes = use_memo(move || update_request(&original(), &form.values()).is_some())();

    let submit = use_callback(move |(): ()| {
        if *saving.peek() || deck.peek().as_ref().is_none_or(Result::is_err) {
            return;
        }
        if let Err(e) = DeckName::new(form.name.peek().clone()) {
            let mut error = form.name_error;
            error.set(Some(e.to_string()));
            return;
        }
        let Some(request) = update_request(&original.peek(), &form.values()) else {
            toast.error(
                InvalidUpdateDeckProfile::NoUpdates.to_string(),
                ToastOptions::default().duration(TOAST_NORMAL),
            );
            return;
        };
        saving.set(true);
        spawn(async move {
            let saved = authed
                .try_run("save_profile", |c, s| async move {
                    c.update_deck_profile(deck_id, &request, &s).await
                })
                .await;
            saving.set(false);
            if saved.is_ok() {
                navigator.push(Route::ViewDeck { deck_id });
            }
        });
    });

    let state = match &*deck.read() {
        Some(Ok(_)) => Some(true),
        Some(Err(_)) => Some(false),
        None => None,
    };
    rsx! {
        ScreenFrame {
            title: "Edit deck",
            description: Some("Change a deck's profile, budget and tags".to_string()),
            on_hint: move |()| {
                let mut hint_open = hint_open;
                hint_open.set(true);
            },
            div { class: "screen-content content-enter",
                match state {
                    Some(true) => rsx! {
                        DeckFields { form, on_enter: move |()| submit.call(()) }
                    },
                    Some(false) => rsx! {
                        p { class: "text-muted", "Could not load deck" }
                    },
                    None => rsx! {
                        EditDeckSkeleton {}
                    },
                }
            }
            ActionBar {
                Button {
                    variant: ButtonVariant::Action,
                    disabled: saving(),
                    onclick: move |_| {
                        navigator.push(Route::ViewDeck { deck_id });
                    },
                    "Back"
                }
                if has_changes {
                    Button {
                        variant: ButtonVariant::Action,
                        disabled: saving(),
                        onclick: move |_| submit.call(()),
                        if saving() {
                            "Saving..."
                        } else {
                            "Save changes"
                        }
                    }
                }
            }
        }
        DeckFormOverlays { form }
        OneTimeHintDialog { hint: OneTimeHint::EditDeck, open: hint_open }
    }
}
