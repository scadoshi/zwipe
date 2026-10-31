//! Create deck: the deck form and `Save`. A commander handed over by
//! Playstyle or the commander maybeboard arrives with the format set to
//! Commander.

use components::{ActionBar, Button, ButtonVariant};
use dioxus::prelude::*;
use zwipe_core::domain::deck::{DeckName, Format};

use super::form::fields::DeckFields;
use super::form::overlays::DeckFormOverlays;
use super::form::use_deck_form;
use crate::domain::deck::create_request;
use crate::inbound::caches::CreateDeckSeed;
use crate::inbound::chrome::authed::use_authed;
use crate::inbound::chrome::one_time_hint::{OneTimeHint, OneTimeHintDialog, use_one_time_hint};
use crate::inbound::router::Route;
use crate::inbound::shell::ScreenFrame;

/// The Create deck screen.
#[component]
pub fn CreateDeck() -> Element {
    let navigator = use_navigator();
    let authed = use_authed();
    let hint_open = use_one_time_hint(OneTimeHint::CreateDeck);
    let form = use_deck_form();
    let mut saving = use_signal(|| false);

    // The handed-over commander, taken once. The format is set directly so
    // the picker's cascade cannot clear the commander.
    let seed: CreateDeckSeed = use_context();
    use_hook(move || {
        let mut slot = seed.0;
        let taken = slot.write().take();
        if let Some(card) = taken {
            let mut format = form.format;
            format.set(Some(Format::Commander));
            form.commander.pick(card);
        }
    });

    let submit = use_callback(move |(): ()| {
        if *saving.peek() {
            return;
        }
        let values = form.values();
        if let Err(e) = DeckName::new(values.name.clone()) {
            let mut error = form.name_error;
            error.set(Some(e.to_string()));
            return;
        }
        saving.set(true);
        let request = create_request(&values);
        spawn(async move {
            match authed
                .try_run("create_deck", |c, s| async move {
                    c.create_deck_profile(&request, &s).await
                })
                .await
            {
                Ok(created) => {
                    navigator.push(Route::ViewDeck {
                        deck_id: created.id,
                    });
                }
                Err(_) => saving.set(false),
            }
        });
    });

    rsx! {
        ScreenFrame {
            title: "Create deck",
            on_hint: move |()| {
                let mut hint_open = hint_open;
                hint_open.set(true);
            },
            div { class: "screen-content content-enter",
                DeckFields { form, on_enter: move |()| submit.call(()) }
            }
            ActionBar {
                Button {
                    variant: ButtonVariant::Action,
                    disabled: saving(),
                    onclick: move |_| navigator.go_back(),
                    "Back"
                }
                Button {
                    variant: ButtonVariant::Action,
                    disabled: saving(),
                    onclick: move |_| submit.call(()),
                    if saving() {
                        "Saving..."
                    } else {
                        "Save"
                    }
                }
            }
        }
        DeckFormOverlays { form }
        OneTimeHintDialog { hint: OneTimeHint::CreateDeck, open: hint_open }
    }
}
